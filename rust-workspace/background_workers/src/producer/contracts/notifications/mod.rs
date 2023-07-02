use crate::producer::Producer;
use notifications::contracts::send_notification::AffectedSubscriberWithLocations;

use crate::producer::contracts::notifications::email::EmailStrategy;
use async_trait::async_trait;

pub mod email;

#[async_trait]
pub trait DeliveryStrategy: Send + Sync {
    async fn deliver(
        &self,
        affected_subscribers: Vec<AffectedSubscriberWithLocations>,
    ) -> anyhow::Result<()>;
}

impl Producer {
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn send_notifications(
        &self,
        locations: Vec<AffectedSubscriberWithLocations>,
    ) -> anyhow::Result<()> {
        // For now, its just email_strategy;;
        let email_strategy = EmailStrategy::new_strategy(self.app.clone());
        email_strategy.deliver(locations).await
    }
}
