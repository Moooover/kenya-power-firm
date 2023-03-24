use std::iter;

use anyhow::Context;
use async_trait::async_trait;
use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;
use use_cases::subscriber_locations::{
    data::LocationInput, subscribe_to_location::SubscribeToLocationRepo,
};
use uuid::Uuid;

use crate::repository::Repository;

impl Repository {
    pub async fn subscribe_to_location(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<Uuid> {
        let subscriber = subscriber.inner();
        let location_id = location_id.inner();
        let _ = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            "#,
            subscriber,
            location_id
        )
        .execute(self.pool())
        .await
        .context("Failed to subscribe to location")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.subscriber_locations WHERE subscriber_id = $1 AND location_id = $2
            "#,
             subscriber,
              location_id
        ).fetch_one(self.pool()).await.context("Failed to get location")?;

        Ok(record.id)
    }

    pub async fn subscribe_to_adjuscent_location(
        &self,
        initial_location_id: Uuid,
        adjuscent_location_id: LocationId,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "
              INSERT INTO location.adjuscent_locations(initial_location_id, adjuscent_location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            ",
            initial_location_id,
            adjuscent_location_id.inner()
        )
        .execute(self.pool())
        .await
        .context("Failed to insert nearby location")?;

        Ok(())
    }
}

#[async_trait]
impl SubscribeToLocationRepo for Repository {
    async fn subscribe(
        &self,
        subscriber: SubscriberId,
        location: LocationInput<LocationId>,
    ) -> anyhow::Result<()> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let subscriber_id = subscriber.inner();

        let record = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING id
            "#,
            subscriber_id,
            location.primary_id().inner()
        )
        .fetch_one(&mut *transaction)
        .await
        .context("Failed to subscribe to location")?;

        let nearby_locations = location
            .nearby_locations
            .into_iter()
            .map(|location| location.inner())
            .collect::<Vec<_>>();
        let take_initial_location = iter::repeat(record.id)
            .take(nearby_locations.len())
            .collect::<Vec<_>>();

        sqlx::query!(
            "
              INSERT INTO location.adjuscent_locations(initial_location_id, adjuscent_location_id) 
              SELECT * FROM UNNEST($1::uuid[], $2::uuid[])
            ",
            &take_initial_location[..],
            &nearby_locations[..],
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to insert nearby locations")?;

        transaction
            .commit()
            .await
            .context("Failed to save location subscription changes")?;

        Ok(())
    }
}
