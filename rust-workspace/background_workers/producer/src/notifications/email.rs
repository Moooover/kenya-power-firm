use anyhow::bail;
use async_trait::async_trait;
use celery::Celery;
use entities::notifications::{DeliveryStrategy, Notification};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::sync::Arc;
use tasks::send_notifications::email::send_email_notification;

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
    async fn deliver(&self, notifications: Vec<Notification>) -> anyhow::Result<()> {
        let mut futures: FuturesUnordered<_> = notifications
            .into_iter()
            .map(|notification| {
                self.app
                    .send_task(send_email_notification::new(notification))
            })
            .collect();

        let mut errors = vec![];
        while let Some(result) = futures.next().await {
            if let Err(e) = result {
                // TODO: Setup logging
                println!("Error sending notification: {e:?}");
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            bail!("There were errors while sending the email notification tasks {errors:?}")
        }

        Ok(())
    }
}
