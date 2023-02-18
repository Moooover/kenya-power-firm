use async_trait::async_trait;
use power_interuptions::location::{AffectedArea, LocationWithDateAndTime};
use subscriptions::subscriber::{AffectedSubscriber, SubscriberId};

pub struct Notification {
    pub subscriber: AffectedSubscriber,
    pub locations: Vec<LocationWithDateAndTime>,
}
#[async_trait]
pub trait DeliveryStrategy: Send + Sync {
    async fn deliver(&self, notifications: Vec<Notification>) -> anyhow::Result<()>;
}
