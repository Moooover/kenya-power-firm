use crate::actor::Actor;
use crate::locations::data::{Location, LocationId, LocationWithId};
use async_trait::async_trait;
use std::sync::Arc;
use subscriptions::subscriber::SubscriberId;

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
}

#[async_trait]
impl SubscribeToLocationInteractor for SubscribeToLocationImpl {
    async fn subscribe(&self, actor: &dyn Actor, location: Location) -> anyhow::Result<()> {
        let id = actor.id();
        let location = self
            .repo
            .create_or_return_existing_location(id, location)
            .await?;
        self.repo.subscribe(id, location.id).await
    }
}
