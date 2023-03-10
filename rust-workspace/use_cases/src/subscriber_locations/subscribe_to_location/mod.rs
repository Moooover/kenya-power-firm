use crate::authentication::subscriber_authentication::SubscriberResolverInteractor;
use crate::subscriber_locations::data::{LocationId, LocationInput, LocationWithId};
use crate::{actor::Actor, search_for_locations::ExternalLocationId};
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait SubscribeToLocationInteractor: Send + Sync {
    async fn subscribe(
        &self,
        actor: &dyn Actor,
        location: LocationInput<ExternalLocationId>,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait SubscribeToLocationRepo: Send + Sync {
    async fn subscribe(
        &self,
        subscriber: SubscriberId,
        locations: LocationInput<LocationId>,
    ) -> anyhow::Result<()>;
}

pub struct SubscribeToLocationImpl {
    repo: Arc<dyn SubscribeToLocationRepo>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    location_details_finder: Arc<dyn LocationDetailsFinder>,
}

impl SubscribeToLocationImpl {
    pub fn new(
        repo: Arc<dyn SubscribeToLocationRepo>,
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
        location_details_finder: Arc<dyn LocationDetailsFinder>,
    ) -> Self {
        Self {
            repo,
            subscriber_resolver,
            location_details_finder,
        }
    }
}

#[async_trait]
pub trait LocationDetailsFinder: Send + Sync {
    async fn location_details(
        &self,
        location: LocationInput<ExternalLocationId>,
    ) -> anyhow::Result<LocationInput<LocationId>>;
}

#[async_trait]
impl SubscribeToLocationInteractor for SubscribeToLocationImpl {
    async fn subscribe(
        &self,
        actor: &dyn Actor,
        location: LocationInput<ExternalLocationId>,
    ) -> anyhow::Result<()> {
        let id = self.subscriber_resolver.resolve_from_actor(actor).await?;
        let location = self
            .location_details_finder
            .location_details(location)
            .await?;
        self.repo.subscribe(id, location).await
    }
}
