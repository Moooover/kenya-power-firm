use crate::actor::Actor;
use crate::locations::data::LocationWithId;
use async_trait::async_trait;

#[async_trait]
pub trait EditLocationInteractor {
    fn edit(
        &self,
        actor: &dyn Actor,
        edited_location: LocationWithId,
    ) -> anyhow::Result<LocationWithId>;
}
