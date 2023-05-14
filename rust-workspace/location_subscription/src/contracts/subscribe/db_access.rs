
use crate::db_access::DbAccess;
use crate::save_and_search_for_locations::{
    AffectedLocation, LocationInput, LocationWithCoordinates, NearbyLocationId,
    SaveAndSearchLocations,
};
use anyhow::Context;
use entities::locations::{ExternalLocationId, LocationId};
use entities::subscriptions::SubscriberId;


use url::Url;


pub(crate) struct SubscriptionDbAccess {
    db: DbAccess,
    save_and_search_for_locations: SaveAndSearchLocations,
}

impl SubscriptionDbAccess {
    pub fn new() -> Self {
        Self {
            db: DbAccess,
            save_and_search_for_locations: SaveAndSearchLocations::new(),
        }
    }
    pub(crate) async fn subscribe(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        let subscriber = subscriber.inner();
        let location_id = location_id.inner();
        let pool = self.db.pool().await;
        let _ = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            "#,
            subscriber,
            location_id
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to insert subscriber_location")?;

        Ok(())
    }

    pub(crate) async fn find_location_by_external_id(
        &self,
        location: ExternalLocationId,
    ) -> anyhow::Result<Option<LocationWithCoordinates>> {
        self.save_and_search_for_locations
            .find_location_coordinates_by_external_id(location)
            .await
    }

    pub(crate) async fn are_nearby_locations_already_saved(
        &self,
        location: LocationId,
    ) -> anyhow::Result<Option<NearbyLocationId>> {
        self.save_and_search_for_locations
            .was_nearby_location_already_saved(location)
            .await
    }

    pub(crate) async fn is_location_affected(
        &self,
        location: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        self.save_and_search_for_locations
            .affected_location(location)
            .await
    }

    pub async fn save_main_location(&self, input: LocationInput) -> anyhow::Result<LocationId> {
        self.save_and_search_for_locations
            .save_main_location(input)
            .await
    }

    pub(super) async fn save_nearby_locations(
        &self,
        url: Url,
        primary_location: LocationId,
        api_response: serde_json::Value,
    ) -> anyhow::Result<NearbyLocationId> {
        self.save_and_search_for_locations
            .save_nearby_locations(url, primary_location, api_response)
            .await
    }
}
