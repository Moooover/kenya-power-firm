use crate::contracts::send_notification::db_access::{
    Notification, NotificationStrategyId, SendNotificationsDbAccess,
};

use crate::db_access::{DbAccess, SourceId};
use anyhow::Context;
use url::Url;


pub struct EmailNotificationsDbAccess {
    db_access: SendNotificationsDbAccess,
}

impl AsRef<DbAccess> for EmailNotificationsDbAccess {
    fn as_ref(&self) -> &DbAccess {
        self.db_access.as_ref()
    }
}

impl EmailNotificationsDbAccess {
    pub fn new() -> Self {
        Self {
            db_access: SendNotificationsDbAccess::new(),
        }
    }

    pub async fn get_email_strategy_id(&self) -> anyhow::Result<NotificationStrategyId> {
        let pool = self.db_access.as_ref().pool().await;
        let strategy_name = "EMAIL";
        let record = sqlx::query!(
            "SELECT id FROM communication.strategies WHERE name = $1",
            strategy_name
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to get strategy id")?;

        Ok(record.id.into())
    }

    pub async fn get_source_by_url(&self, url: &Url) -> anyhow::Result<SourceId> {
        self.db_access.get_source_by_url(url).await
    }

    pub async fn save_email_notification_sent(
        &self,
        notification: impl Notification,
        external_id: String,
    ) -> anyhow::Result<()> {
        let strategy = self.get_email_strategy_id().await?;
        let source = self.get_source_by_url(&notification.url()).await?;
        self.db_access
            .save_notification_sent(notification, strategy, source, external_id)
            .await
    }
}
