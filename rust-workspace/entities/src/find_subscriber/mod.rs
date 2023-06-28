use crate::subscriptions::details::{
    SubscriberDetails, SubscriberEmail, SubscriberExternalId, SubscriberName,
};

pub struct FindSubscriber {
    db: DbAccess,
}

use crate::db_access::DbAccess;
use anyhow::{anyhow, Context};
use shared_kernel::subscriber_id::SubscriberId;

impl FindSubscriber {
    pub fn new() -> Self {
        Self { db: DbAccess {} }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn find_by_external_id(
        &self,
        external_id: SubscriberExternalId,
    ) -> anyhow::Result<SubscriberId> {
        let pool = self.db.pool().await;
        let result = sqlx::query!(
            "
            SELECT id FROM public.subscriber WHERE external_id = $1
            ",
            external_id.as_ref()
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch subscriber details")?;

        Ok(result.id.into())
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn find_subscriber_by_id(
        &self,
        id: SubscriberId,
    ) -> anyhow::Result<SubscriberDetails> {
        let id = id.inner();
        let pool = self.db.pool().await;
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
}
