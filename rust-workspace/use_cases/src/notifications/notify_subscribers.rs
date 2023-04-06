use crate::import_affected_areas::NotifySubscribersOfAffectedAreas;
use async_trait::async_trait;
use entities::notifications::{DeliveryStrategy, Notification};
use entities::power_interruptions::location::{
    AffectedLine, ImportInput, NairobiTZDateTime, Region,
};
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

pub struct Notifier {
    subscriber_repo: Arc<dyn SubscriberRepo>,
    subscriber_delivery_strategies: Arc<dyn GetPreferredDeliveryStrategies>,
}

#[derive(Clone)]
pub struct SubscriberWithAffectedLines {
    subscriber: AffectedSubscriber,
    lines: Vec<AffectedLine<NairobiTZDateTime>>,
}

#[async_trait]
pub trait SubscriberRepo: Send + Sync {
    async fn get_affected_subscribers(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine<NairobiTZDateTime>>>>;
}

pub struct StrategyWithSubscribers {
    pub strategy: Arc<dyn DeliveryStrategy>,
    pub subscribers: Vec<SubscriberId>,
}

#[async_trait]
pub trait GetPreferredDeliveryStrategies: Send + Sync {
    async fn get_strategies(
        &self,
        subscribers: Vec<SubscriberId>,
    ) -> anyhow::Result<Vec<StrategyWithSubscribers>>;
}

#[async_trait]
impl NotifySubscribersOfAffectedAreas for Notifier {
    async fn notify(&self, data: ImportInput) -> anyhow::Result<()> {
        let mut futures: FuturesUnordered<_> = data
            .0
            .iter()
            .map(|(url, regions)| self.notify_affected_subscribers(url.clone(), regions))
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
    pub fn new(
        subscriber_repo: Arc<dyn SubscriberRepo>,
        subscriber_delivery_strategies: Arc<dyn GetPreferredDeliveryStrategies>,
    ) -> Self {
        Self {
            subscriber_repo,
            subscriber_delivery_strategies,
        }
    }
    async fn notify_affected_subscribers(
        &self,
        url: Url,
        regions: &[Region],
    ) -> anyhow::Result<()> {
        let mapping_of_subscriber_to_locations = self
            .subscriber_repo
            .get_affected_subscribers(regions)
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
            .map(|(subscriber, lines)| {
                (
                    subscriber.id(),
                    SubscriberWithAffectedLines { subscriber, lines },
                )
            })
            .collect::<HashMap<_, _>>();

        let mut notification_futures: FuturesUnordered<_> = strategies_with_subscribers
            .iter()
            .map(
                |StrategyWithSubscribers {
                     strategy,
                     subscribers,
                 }| {
                    let notifications = subscribers
                        .iter()
                        .filter_map(|subscriber| {
                            mapping_of_subscriber_to_locations
                                .get(subscriber)
                                .cloned()
                                .map(|data| Notification {
                                    url: url.clone(),
                                    subscriber: data.subscriber,
                                    lines: data.lines,
                                })
                        })
                        .collect::<Vec<_>>();
                    strategy.deliver(notifications)
                },
            )
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
