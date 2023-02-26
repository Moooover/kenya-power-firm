use crate::actor::Actor;
use crate::authentication::subscriber::SubscriberResolverInteractor;
use crate::locations::data::LocationWithId;
use async_trait::async_trait;
use std::sync::Arc;
use subscriptions::subscriber::SubscriberId;
use uuid::Uuid;

#[async_trait]
pub trait ListSubscribedLocations: Send + Sync {
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<LocationWithId>>;
}

pub struct ListSubscribedLocationsImpl {
    repo: Arc<dyn LocationsSubscribedRepo>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
}

impl ListSubscribedLocationsImpl {
    pub fn new(
        repo: Arc<dyn LocationsSubscribedRepo>,
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    ) -> Self {
        Self {
            repo,
            subscriber_resolver,
        }
    }
}

#[async_trait]
pub trait LocationsSubscribedRepo: Send + Sync {
    async fn list(&self, id: SubscriberId) -> anyhow::Result<Vec<LocationWithId>>;
}

#[async_trait]
impl ListSubscribedLocations for ListSubscribedLocationsImpl {
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<LocationWithId>> {
        let id = self.subscriber_resolver.resolve_from_actor(actor).await?;

        self.repo.list(id).await
    }
}
