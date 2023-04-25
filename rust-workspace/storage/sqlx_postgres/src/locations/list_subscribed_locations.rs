use std::collections::HashMap;

use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use itertools::Itertools;
use use_cases::subscriber_locations::list_subscribed_locations::{
    LocationWithId, LocationsSubscribedRepo,
};

#[async_trait]
impl LocationsSubscribedRepo for Repository {
    async fn list(&self, id: SubscriberId) -> anyhow::Result<Vec<LocationWithId>> {
        struct BareLocationDetails {
            name: String,
            address: String,
        }

        let pool = self.pool();
        let id = id.inner();
        let primary_locations = sqlx::query!(
            "
            SELECT id, location_id FROM location.subscriber_locations WHERE subscriber_id = $1
            ",
            id
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch primary locations")?;

        let primary_location_ids: Vec<_> = primary_locations
            .iter()
            .map(|result| result.location_id)
            .collect();

        let location_details = sqlx::query!(
            "
            SELECT id, name, address FROM location.locations WHERE id = ANY($1)
            ",
            &primary_location_ids[..]
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch location details")?;

        let mapping_of_location_id_to_details = location_details
            .into_iter()
            .map(|location| {
                (
                    location.id,
                    BareLocationDetails {
                        name: location.name,
                        address: location.address,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let locations = primary_locations
            .iter()
            .filter_map(|primary_location| {
                mapping_of_location_id_to_details
                    .get(&primary_location.location_id)
                    .map(|location| LocationWithId {
                        id: primary_location.location_id.into(),
                        name: location.name.to_owned(),
                        address: location.address.to_owned(),
                    })
            })
            .collect_vec();

        Ok(locations)
    }
}
