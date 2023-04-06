use crate::authentication::subscriber_authentication::SubscriberResolverInteractor;
use crate::subscriber_locations::data::{LocationInput};
use crate::{actor::Actor, search_for_locations::Status};

use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;
use shared_kernel::string_key;
use std::sync::Arc;

string_key!(TaskId);

#[async_trait]
pub trait SubscribeToLocationInteractor: Send + Sync {
    async fn subscribe(
        &self,
        actor: &dyn Actor,
        location: LocationInput<String>,
    ) -> anyhow::Result<TaskId>;

    async fn progress(&self, actor: &dyn Actor, task_id: TaskId) -> anyhow::Result<Status>;
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
    ) -> anyhow::Result<TaskId>;

    async fn progress(&self, task_id: TaskId) -> anyhow::Result<Status>;
}

#[async_trait]
impl SubscribeToLocationInteractor for SubscribeToLocationImpl {
    async fn subscribe(
        &self,
        actor: &dyn Actor,
        location: LocationInput<String>,
    ) -> anyhow::Result<TaskId> {
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

    async fn progress(&self, actor: &dyn Actor, task_id: TaskId) -> anyhow::Result<Status> {
        // we just want to ensure that the user is valid
        let _ = self.subscriber_resolver.resolve_from_actor(actor).await?;
        self.location_subscriber.progress(task_id).await
    }
}
