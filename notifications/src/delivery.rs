use async_trait::async_trait;
use power_interuptions::location::LocationWithDateAndTime;
use subscriptions::subscriber::{AffectedSubscriber, SubscriberId};
use use_cases::import_planned_blackouts::Url;

pub struct Notification {
    pub url: Url,
    pub subscriber: AffectedSubscriber,
    pub locations: Vec<LocationWithDateAndTime>,
}
#[async_trait]
pub trait DeliveryStrategy: Send + Sync {
    async fn deliver(&self, notifications: Vec<Notification>) -> anyhow::Result<()>;
}
