use crate::delivery::{DeliveryStrategy, Notification};
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use power_interuptions::location::{ImportInput, LocationWithDateAndTime, Region};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use subscriber::subscriber::{AffectedSubscriber, SubscriberId};
use url::Url;
use use_cases::import_planned_blackouts::NotifySubscribersOfAffectedAreas;

pub struct Notifier {
    subscriber_repo: Arc<dyn SubscriberRepo>,
    subscriber_delivery_strategies: Arc<dyn GetPreferredDeliveryStrategies>,
}

#[derive(Clone)]
pub struct SubscriberWithLocations {
    subscriber: AffectedSubscriber,
    locations: Vec<LocationWithDateAndTime>,
}

#[async_trait]
pub trait SubscriberRepo: Send + Sync {
    async fn get_subscribers_from_affected_locations(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<LocationWithDateAndTime>>>;
}

#[async_trait]
pub trait GetPreferredDeliveryStrategies: Send + Sync {
    async fn get_strategies(
        &self,
        subscribers: Vec<SubscriberId>,
    ) -> anyhow::Result<HashMap<Arc<dyn DeliveryStrategy>, Vec<SubscriberId>>>;
}

#[async_trait]
impl NotifySubscribersOfAffectedAreas for Notifier {
    async fn notify(&self, data: ImportInput) -> anyhow::Result<()> {
        let mut futures: FuturesUnordered<_> = data
            .0
            .iter()
            .map(|(url, regions)| self.notify_subscribers_in_regions(url.clone(), regions))
            .collect();

        while let Some(result) = futures.next().await {
            if let Err(e) = result {
                // TODO: Setup logging
                // error!("Error sending notification: {:?}", e);
                println!("Error notifying subscribers: {e:?}")
            }
        }

        Ok(())
    }
}

impl Notifier {
    async fn notify_subscribers_in_regions(
        &self,
        url: Url,
        regions: &[Region],
    ) -> anyhow::Result<()> {
        let mapping_of_subscriber_to_locations = self
            .subscriber_repo
            .get_subscribers_from_affected_locations(regions)
            .await?;

        let subscriber_ids = mapping_of_subscriber_to_locations
            .keys()
            .map(|key| key.id())
            .collect::<Vec<_>>();

        let strategies_with_subscribers = self
            .subscriber_delivery_strategies
            .get_strategies(subscriber_ids)
            .await?;

        let mapping_of_subscriber_to_locations = mapping_of_subscriber_to_locations
            .into_iter()
            .map(|(subscriber, locations)| {
                (
                    subscriber.id(),
                    SubscriberWithLocations {
                        subscriber,
                        locations,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let mut notification_futures: FuturesUnordered<_> = strategies_with_subscribers
            .iter()
            .map(|(strategy, subscribers)| {
                let notifications = subscribers
                    .iter()
                    .filter_map(|subscriber| {
                        mapping_of_subscriber_to_locations
                            .get(subscriber)
                            .cloned()
                            .map(|data| Notification {
                                url: url.clone(),
                                subscriber: data.subscriber,
                                locations: data.locations,
                            })
                    })
                    .collect::<Vec<_>>();
                strategy.deliver(notifications)
            })
            .collect();

        while let Some(result) = notification_futures.next().await {
            if let Err(e) = result {
                // TODO: Setup logging
                // error!("Error sending notification: {:?}", e);
                println!("Error sending notifications: {e:?}")
            }
        }

        Ok(())
    }
}
