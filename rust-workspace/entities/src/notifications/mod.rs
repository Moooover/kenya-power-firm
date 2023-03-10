use crate::power_interruptions::location::AffectedLine;
use crate::subscriptions::AffectedSubscriber;
use async_trait::async_trait;
use url::Url;

pub struct Notification {
    pub url: Url,
    pub subscriber: AffectedSubscriber,
    pub lines: Vec<AffectedLine>,
}
#[async_trait]
pub trait DeliveryStrategy: Send + Sync {
    async fn deliver(&self, notifications: Vec<Notification>) -> anyhow::Result<()>;
}
