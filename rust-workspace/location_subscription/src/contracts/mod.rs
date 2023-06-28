use async_trait::async_trait;
use shared_kernel::location_ids::LocationId;
use shared_kernel::subscriber_id::SubscriberId;
use use_cases::subscriber_locations::delete_locations_subscribed_to::DeleteSubscribedLocationOp;
use use_cases::subscriber_locations::list_subscribed_locations::{
    ListSubscribedLocationsOp, LocationWithId,
};

pub mod get_affected_subscribers_from_import;
pub mod get_currently_affected_subscribers;
pub mod import_locations_to_search_engine;
pub mod list_subscribed_locations;
pub mod subscribe;
pub mod unsubscribe;

#[derive(Clone)]
pub struct LocationSubscriptionSubSystem;

#[async_trait]
impl DeleteSubscribedLocationOp for LocationSubscriptionSubSystem {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn delete_subscribed(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        unsubscribe::UnsubscribeFromLocationInteractor::unsubscribe_from_location(
            subscriber_id,
            location_id,
        )
        .await
    }
}

#[async_trait]
impl ListSubscribedLocationsOp for LocationSubscriptionSubSystem {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn list(&self, id: SubscriberId) -> anyhow::Result<Vec<LocationWithId>> {
        let list = list_subscribed_locations::ListSubscribedLocationsInteractor::list_subscribed_locations(id).await?;

        Ok(list
            .into_iter()
            .map(|l| LocationWithId {
                id: l.id,
                name: l.name.to_string(),
                address: l.address,
            })
            .collect())
    }
}
