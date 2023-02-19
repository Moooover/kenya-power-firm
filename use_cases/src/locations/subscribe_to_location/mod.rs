use crate::actor::Actor;
use crate::locations::data::Location;
use async_trait::async_trait;

#[async_trait]
pub trait SubscribeToLocationInteractor {
    async fn subscribe(&self, actor: &dyn Actor, location: Location) -> anyhow::Result<()>;
}
