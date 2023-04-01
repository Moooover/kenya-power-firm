use std::collections::HashMap;

use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use itertools::Itertools;
use use_cases::subscriber_locations::data::{AdjuscentLocation, LocationWithId};
use use_cases::subscriber_locations::list_subscribed_locations::LocationsSubscribedRepo;

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

        let (primary_location_ids, initial_location_id): (Vec<_>, Vec<_>) = primary_locations
            .iter()
            .map(|result| (result.location_id, result.id))
            .multiunzip();

        let adjuscent_locations = sqlx::query!(
            "
            SELECT id, initial_location_id, adjuscent_location_id FROM location.adjuscent_locations WHERE initial_location_id = ANY($1)
            ",
            &initial_location_id[..]
        ).fetch_all(pool).await.context("Failed to fetch adjuscent locations")?;

        let adjuscent_location_ids = adjuscent_locations
            .iter()
            .map(|location| location.adjuscent_location_id);

        let primary_and_adjuscent_location_ids = adjuscent_location_ids
            .chain(primary_location_ids.into_iter())
            .collect_vec();

        let location_details = sqlx::query!(
            "
            SELECT id, name, address FROM location.locations WHERE id = ANY($1)
            ",
            &primary_and_adjuscent_location_ids[..]
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

        let mapping_of_initial_location_id_to_adjuscent_locations = adjuscent_locations
            .iter()
            .into_group_map_by(|location| location.initial_location_id);

        let empty_vec = vec![];

        let locations = primary_locations
            .iter()
            .filter_map(|primary_location| {
                mapping_of_location_id_to_details
                    .get(&primary_location.location_id)
                    .map(|location| {
                        let adjuscent_locations =
                            mapping_of_initial_location_id_to_adjuscent_locations
                                .get(&primary_location.id)
                                .unwrap_or(&empty_vec);

                        let adjuscent_locations = adjuscent_locations
                            .iter()
                            .filter_map(|adjuscent_location| {
                                let details = mapping_of_location_id_to_details
                                    .get(&adjuscent_location.adjuscent_location_id);

                                details.map(|details| AdjuscentLocation {
                                    id: adjuscent_location.adjuscent_location_id.into(),
                                    name: details.name.clone(),
                                    address: details.address.clone(),
                                })
                            })
                            .collect_vec();

                        LocationWithId {
                            id: primary_location.location_id.into(),
                            name: location.name.to_owned(),
                            address: location.address.to_owned(),
                            adjuscent_locations,
                        }
                    })
            })
            .collect_vec();

        Ok(locations)
    }
}
