use async_trait::async_trait;
use power_interuptions::location::AffectedArea;
use subscriptions::subscriber::SubscriberId;

pub struct Notification {
    subscriber: SubscriberId,
    areas_affected: Vec<AffectedArea>,
}
#[async_trait]
pub trait DeliveryStrategy: Send + Sync {
    async fn deliver(&self, notifications: Vec<Notification>);
}
