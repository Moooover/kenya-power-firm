use std::iter;

use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use use_cases::subscriber_locations::{
    data::{LocationId, LocationInput},
    subscribe_to_location::SubscribeToLocationRepo,
};

use crate::repository::Repository;

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

        let subscriber_id = subscriber.into_inner();

        let record = sqlx::query!(
            r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id) 
              VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING id
            "#,
            subscriber_id,
            location.primary_id().into_inner()
        )
        .fetch_one(&mut *transaction)
        .await
        .context("Failed to subscribe to location")?;

        let nearby_locations = location
            .nearby_locations
            .into_iter()
            .map(|location| location.into_inner())
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
