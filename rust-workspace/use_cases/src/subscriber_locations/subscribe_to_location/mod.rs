use crate::actor::Actor;
use crate::authentication::subscriber_authentication::SubscriberResolverInteractor;
use crate::subscriber_locations::data::{LocationInput, LocationWithId};
use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait SubscribeToLocationInteractor: Send + Sync {
    async fn subscribe(
        &self,
        actor: &dyn Actor,
        location: LocationInput<String>,
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
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    location_subscriber: Arc<dyn LocationSubscriber>,
}

impl SubscribeToLocationImpl {
    pub fn new(
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
        location_subscriber: Arc<dyn LocationSubscriber>,
    ) -> Self {
        Self {
            subscriber_resolver,
            location_subscriber,
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
pub trait LocationSubscriber: Send + Sync {
    async fn subscribe_to_location(
        &self,
        location: LocationInput<ExternalLocationId>,
        subscriber: SubscriberId,
    ) -> anyhow::Result<()>;
}

#[async_trait]
impl SubscribeToLocationInteractor for SubscribeToLocationImpl {
    async fn subscribe(
        &self,
        actor: &dyn Actor,
        location: LocationInput<String>,
    ) -> anyhow::Result<()> {
        let id = self.subscriber_resolver.resolve_from_actor(actor).await?;
        let location = LocationInput {
            id: ExternalLocationId::new(location.id),
            nearby_locations: location
                .nearby_locations
                .into_iter()
                .map(ExternalLocationId::new)
                .collect(),
        };
        self.location_subscriber
            .subscribe_to_location(location, id)
            .await
    }
}
