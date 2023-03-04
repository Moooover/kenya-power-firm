use crate::actor::Actor;
use crate::authentication::subscriber_authentication::SubscriberResolverInteractor;
use crate::locations::data::{Location, LocationId, LocationWithId};
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use std::sync::Arc;

#[async_trait]
pub trait SubscribeToLocationInteractor: Send + Sync {
    async fn subscribe(&self, actor: &dyn Actor, location: Location) -> anyhow::Result<()>;
}

#[async_trait]
pub trait CreateLocationRepo: Send + Sync {
    async fn create_or_return_existing_location(
        &self,
        subscriber_id: SubscriberId,
        location: Location,
    ) -> anyhow::Result<LocationWithId>;
}

#[async_trait]
pub trait SubscribeToLocationRepo: Send + Sync {
    async fn subscribe(&self, subscriber: SubscriberId, location: LocationId)
        -> anyhow::Result<()>;
}

pub trait CreateLocationAndSubscribeRepo: SubscribeToLocationRepo + CreateLocationRepo {}

pub struct SubscribeToLocationImpl {
    repo: Arc<dyn CreateLocationAndSubscribeRepo>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
}

impl SubscribeToLocationImpl {
    pub fn new(
        repo: Arc<dyn CreateLocationAndSubscribeRepo>,
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    ) -> Self {
        Self {
            repo,
            subscriber_resolver,
        }
    }
}

#[async_trait]
impl SubscribeToLocationInteractor for SubscribeToLocationImpl {
    async fn subscribe(&self, actor: &dyn Actor, location: Location) -> anyhow::Result<()> {
        let id = self.subscriber_resolver.resolve_from_actor(actor).await?;
        let location = self
            .repo
            .create_or_return_existing_location(id, location)
            .await?;
        self.repo.subscribe(id, location.id).await
    }
}
