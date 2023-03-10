use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use use_cases::{actor::Actor, authentication::subscriber_authentication::SubscriberResolverRepo};

#[async_trait]
impl SubscriberResolverRepo for Repository {
    async fn find(&self, actor: &dyn Actor) -> anyhow::Result<SubscriberId> {
        let external_id = actor.external_id();
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
}
