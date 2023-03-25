use crate::repository::Repository;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use entities::subscriptions::details::{SubscriberDetails, SubscriberEmail, SubscriberName};
use entities::subscriptions::{details::SubscriberExternalId, SubscriberId};
use use_cases::{actor::Actor, authentication::subscriber_authentication::SubscriberResolverRepo};

#[async_trait]
impl SubscriberResolverRepo for Repository {
    async fn find(&self, actor: &dyn Actor) -> anyhow::Result<SubscriberId> {
        let external_id = actor.external_id();
        self.find_by_external_id(external_id).await
    }
}

impl Repository {
    pub async fn find_by_external_id(
        &self,
        external_id: SubscriberExternalId,
    ) -> anyhow::Result<SubscriberId> {
        let result = sqlx::query!(
            "
            SELECT id FROM public.subscriber WHERE external_id = $1
            ",
            external_id.as_ref()
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to fetch subscriber details")?;

        Ok(result.id.into())
    }

    pub async fn find_subscriber_by_id(&self, id: SubscriberId) -> anyhow::Result<SubscriberDetails> {
        let id = id.inner();
        let result = sqlx::query!(
            "
            SELECT * FROM public.subscriber WHERE id = $1
            ",
            id
        )
        .fetch_one(self.pool())
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
