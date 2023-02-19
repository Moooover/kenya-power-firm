use crate::actor::Actor;
use crate::locations::data::LocationWithId;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ListSubscribedLocations {
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<LocationWithId>>;
}
