use crate::config::SETTINGS_CONFIG;
use anyhow::{anyhow, Context};
use async_once::AsyncOnce;
use entities::locations::{LocationDetails, LocationId, LocationName};
use entities::subscriptions::details::{
    SubscriberDetails, SubscriberEmail, SubscriberExternalId, SubscriberName,
};
use entities::subscriptions::SubscriberId;
use itertools::Itertools;
use lazy_static::lazy_static;
use shared_kernel::uuid_key;
use sqlx_postgres::pool_manager::{PoolManager, PoolWrapper};
use std::collections::{HashMap, HashSet};
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

    pub async fn find_subscriber_by_id(
        &self,
        id: SubscriberId,
    ) -> anyhow::Result<SubscriberDetails> {
        let id = id.inner();
        let pool = self.pool().await;
        let result = sqlx::query!(
            "
            SELECT * FROM public.subscriber WHERE id = $1
            ",
            id
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch subscriber details")?;
        let name = SubscriberName::try_from(result.name).map_err(|err| anyhow::anyhow!(err))?;
        let email = SubscriberEmail::try_from(result.email).map_err(|err| anyhow!(err))?;
        let external_id =
            SubscriberExternalId::try_from(result.external_id).map_err(|err| anyhow!(err))?;
        let subscriber = SubscriberDetails {
            name,
            email,
            external_id,
        };
        Ok(subscriber)
    }

    pub async fn get_locations_by_ids(
        &self,
        ids: HashSet<LocationId>,
    ) -> anyhow::Result<HashMap<LocationId, LocationDetails>> {
        let pool = self.pool().await;
        let ids = ids.into_iter().map(|id| id.inner()).collect_vec();
        let results = sqlx::query!(
            "
            SELECT id, name FROM location.locations WHERE id = ANY($1)
            ",
            &ids[..]
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to fetch locations")?;

        let results = results
            .into_iter()
            .map(|record| {
                (
                    LocationId::from(record.id),
                    LocationDetails {
                        id: LocationId::from(record.id),
                        name: LocationName::from(record.name),
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        Ok(results)
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
