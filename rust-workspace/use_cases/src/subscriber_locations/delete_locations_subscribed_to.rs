use std::sync::Arc;

use crate::{
    actor::Actor, authentication::subscriber_authentication::SubscriberResolverInteractor,
};
use async_trait::async_trait;
use entities::{locations::LocationId, subscriptions::SubscriberId};

#[async_trait]
pub trait DeleteLocationsSubscribedToInteractor {
    async fn delete_primary_location(
        &self,
        actor: &dyn Actor,
        location_id: LocationId,
    ) -> anyhow::Result<()>;
    async fn delete_adjuscent_location(
        &self,
        actor: &dyn Actor,
        location_id: LocationId,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait DeleteSubscribedLocationsRepo: Send + Sync {
    async fn delete_primary_location(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()>;
    async fn delete_adjuscent_location(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()>;
}

pub struct DeleteLocationsSubscribedToImpl {
    repo: Arc<dyn DeleteSubscribedLocationsRepo>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
}

impl DeleteLocationsSubscribedToImpl {
    pub fn new(
        repo: Arc<dyn DeleteSubscribedLocationsRepo>,
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    ) -> Self {
        Self {
            repo,
            subscriber_resolver,
        }
    }
}

#[async_trait]
impl DeleteLocationsSubscribedToInteractor for DeleteLocationsSubscribedToImpl {
    async fn delete_primary_location(
        &self,
        actor: &dyn Actor,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        let subscriber_id = self.subscriber_resolver.resolve_from_actor(actor).await?;
        self.repo
            .delete_primary_location(subscriber_id, location_id)
            .await
    }
    async fn delete_adjuscent_location(
        &self,
        actor: &dyn Actor,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        let subscriber_id = self.subscriber_resolver.resolve_from_actor(actor).await?;
        self.repo
            .delete_adjuscent_location(subscriber_id, location_id)
            .await
    }
}
