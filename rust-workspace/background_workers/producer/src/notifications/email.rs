use crate::notifications::DeliveryStrategy;
use anyhow::bail;
use async_trait::async_trait;
use celery::Celery;

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use notifications::contracts::send_notification::AffectedSubscriberWithLocations;
use std::sync::Arc;
use tasks::tasks::send_notifications::email::send_email_notification;
use tracing::error;

pub struct EmailStrategy {
    pub(crate) app: Arc<Celery>,
}

impl EmailStrategy {
    pub(crate) fn new_strategy(app: Arc<Celery>) -> Arc<dyn DeliveryStrategy> {
        let strategy = EmailStrategy { app };
        Arc::new(strategy)
    }
}

#[async_trait]
impl DeliveryStrategy for EmailStrategy {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn deliver(&self, locations: Vec<AffectedSubscriberWithLocations>) -> anyhow::Result<()> {
        let mut futures: FuturesUnordered<_> = locations
            .into_iter()
            .map(|location| self.app.send_task(send_email_notification::new(location)))
            .collect();

        let mut errors = vec![];
        while let Some(result) = futures.next().await {
            if let Err(e) = result {
                error!("Error sending notification: {e:?}");
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            bail!("There were errors while sending the email notification tasks {errors:?}")
        }

        Ok(())
    }
}
