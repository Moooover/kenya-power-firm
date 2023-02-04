use crate::delivery::DeliveryStrategy;
use async_trait::async_trait;
use power_interuptions::location::{AffectedArea, AreaId};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use subscriptions::subscriber::SubscriberId;
use use_cases::import_planned_blackouts::NotifySubscribersOfAffectedAreas;

pub struct Notifier {
    subscriber_repo: Arc<dyn SubscriberRepo>,
    subscriber_delivery_strategies: Arc<dyn GetPrefferedDeliveryStrategies>,
}

#[async_trait]
pub trait SubscriberRepo: Send + Sync {
    async fn get_subscribers_from_affected_areas(
        &self,
        areas: &[AreaId],
    ) -> Result<HashMap<AreaId, Vec<SubscriberId>>, Box<dyn Error>>;
}

#[async_trait]
pub trait GetPrefferedDeliveryStrategies: Send + Sync {
    async fn get_strategies(
        &self,
        subscribers: &[SubscriberId],
    ) -> Result<HashMap<Arc<dyn DeliveryStrategy>, Vec<SubscriberId>>, Box<dyn Error>>;
}

#[async_trait]
impl NotifySubscribersOfAffectedAreas for Notifier {
    async fn notify(&self, data: Vec<AffectedArea>) -> Result<(), Box<dyn Error>> {
        let subscribers = self
            .subscriber_repo
            .get_subscribers_from_affected_areas(&[])
            .await?;
        let strategies_with_subscribers = self
            .subscriber_delivery_strategies
            .get_strategies(&[])
            .await?;

        // TODO: Improve this mechanism
        for (strategy, subscribers) in strategies_with_subscribers.iter() {
            // get all messages for a the strategy
            strategy.deliver(vec![]).await
        }

        todo!()
    }
}
