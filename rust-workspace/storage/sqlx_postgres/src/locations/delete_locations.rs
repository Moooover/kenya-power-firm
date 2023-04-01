use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::{locations::LocationId, subscriptions::SubscriberId};
use use_cases::subscriber_locations::delete_locations_subscribed_to::DeleteSubscribedLocationsRepo;

#[async_trait]
impl DeleteSubscribedLocationsRepo for Repository {
    async fn delete_primary_location(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        let subscriber_id = subscriber_id.inner();
        let location_id = location_id.inner();
        let _ = sqlx::query!(
            "DELETE FROM location.subscriber_locations 
            WHERE subscriber_id = $1 AND location_id = $2",
            subscriber_id,
            location_id
        )
        .execute(self.pool())
        .await
        .context("Failed to delete location")?;

        Ok(())
    }
    async fn delete_adjuscent_location(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        let subscriber_id = subscriber_id.inner();
        let location_id = location_id.inner();
        let _ = sqlx::query!(
            "
            DELETE FROM location.adjuscent_locations a 
            WHERE a.adjuscent_location_id = $1 
            AND a.initial_location_id IN (select id FROM location.subscriber_locations WHERE subscriber_id = $2);
            ", location_id, subscriber_id
        ).execute(self.pool()).await.context("Failed to delete adjuscent_location")?;

        Ok(())
    }
}
