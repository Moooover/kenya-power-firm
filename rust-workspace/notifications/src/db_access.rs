use crate::config::SETTINGS_CONFIG;
use anyhow::Context;
use async_once::AsyncOnce;
use lazy_static::lazy_static;
use shared_kernel::location_ids::LocationId;
use shared_kernel::subscriber_id::SubscriberId;
use shared_kernel::uuid_key;
use sqlx_postgres::pool_manager::{PoolManager, PoolWrapper};
use std::collections::HashSet;
use url::Url;
use uuid::Uuid;

lazy_static! {
    static ref POOL_MANAGER: AsyncOnce<PoolManager> = AsyncOnce::new(async {
        PoolManager::new(SETTINGS_CONFIG.database.notification_connections)
            .await
            .expect("PoolManager not initialized")
    });
}

#[derive(Copy, Clone)]
pub struct DbAccess;

uuid_key!(SourceId);

impl DbAccess {
    pub async fn pool(&self) -> PoolWrapper<'static> {
        POOL_MANAGER.get().await.pool()
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub(crate) async fn get_source_by_url(&self, url: &Url) -> anyhow::Result<SourceId> {
        let pool = self.pool().await;
        let source = sqlx::query!(
            "SELECT id FROM public.source WHERE url = $1",
            url.to_string()
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to get source")?;

        Ok(source.id.into())
    }
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DbNotificationIdempotencyKey {
    pub source_id: Uuid,
    pub subscriber_id: Uuid,
    pub line: String,
    pub strategy_id: Uuid,
}

impl DbNotificationIdempotencyKey {
    #[tracing::instrument(skip(db), level = "debug")]
    pub async fn get_already_send_notifications(
        db: impl AsRef<DbAccess>,
        strategy_id: Uuid,
        subscriber_id: SubscriberId,
        lines: Vec<String>,
        source_id: SourceId,
    ) -> anyhow::Result<HashSet<Self>> {
        let pool = db.as_ref().pool().await;
        let notifications = sqlx::query!(
            "SELECT source_id, subscriber_id, line, strategy_id FROM communication.notifications 
            WHERE source_id = $1 AND subscriber_id = $2 AND line = ANY($3) AND strategy_id = $4",
            source_id.inner(),
            subscriber_id.inner(),
            &lines[..],
            strategy_id
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to get already send notifications")?;

        Ok(notifications
            .into_iter()
            .map(|record| DbNotificationIdempotencyKey {
                source_id: record.source_id,
                subscriber_id: record.subscriber_id,
                line: record.line,
                strategy_id: record.strategy_id,
            })
            .collect())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LocationDetails {
    pub id: LocationId,
    pub name: String,
}
