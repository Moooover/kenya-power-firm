use crate::actor::Actor;
use crate::locations::data::{Location, LocationId, LocationWithId};
use crate::locations::subscribe_to_location::CreateLocationRepo;
use async_trait::async_trait;
use std::sync::Arc;
use subscriptions::subscriber::SubscriberId;
use thiserror::Error;

#[async_trait]
pub trait EditLocationInteractor: Send + Sync {
    async fn edit(
        &self,
        actor: &dyn Actor,
        location_change: LocationChange<Location>,
    ) -> Result<LocationWithId, EditLocationError>;
}

pub struct LocationChange<T> {
    pub old_location: LocationId,
    pub new_location: T,
}

#[derive(Debug, Error)]
pub enum EditLocationError {
    #[error("{0}")]
    Validation(String),
    #[error("internal server error")]
    InternalServerError(#[from] anyhow::Error),
}

#[async_trait]
pub trait EditLocationRepo: Send + Sync {
    async fn edit(
        &self,
        subscriber: SubscriberId,
        location_change: LocationChange<LocationId>,
    ) -> Result<LocationWithId, EditLocationError>;
}

#[async_trait]
pub trait CreateAndEditLocationRepo: EditLocationRepo + CreateLocationRepo {}

pub struct EditLocationInteractorImpl {
    location_repo: Arc<dyn CreateAndEditLocationRepo>,
}

#[async_trait]
impl EditLocationInteractor for EditLocationInteractorImpl {
    async fn edit(
        &self,
        actor: &dyn Actor,
        location_change: LocationChange<Location>,
    ) -> Result<LocationWithId, EditLocationError> {
        let id = actor.id();
        let new_location = self
            .location_repo
            .create_or_return_existing_location(id, location_change.new_location)
            .await
            .map_err(EditLocationError::InternalServerError)?;
        let change = LocationChange {
            old_location: location_change.old_location,
            new_location: new_location.id,
        };
        self.location_repo.edit(id, change).await
    }
}
