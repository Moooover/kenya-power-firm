use crate::data_transfer::LocationDetails;
use crate::db_access::DbAccess;
use anyhow::Context;
use entities::subscriptions::SubscriberId;
use itertools::Itertools;
use std::collections::HashMap;

pub struct SubscribedLocationsAccess {
    db_access: DbAccess,
}

impl SubscribedLocationsAccess {
    pub fn new() -> Self {
        Self {
            db_access: DbAccess,
        }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn get_subscribed_locations(
        &self,
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<Vec<LocationDetails>> {
        struct BareLocationDetails {
            name: String,
            address: String,
        }

        let pool = self.db_access.pool().await;
        let id = subscriber_id.inner();
        let primary_locations = sqlx::query!(
            "
            SELECT id, location_id FROM location.subscriber_locations WHERE subscriber_id = $1
            ",
            id
        )
        .fetch_all(pool.as_ref())
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
        .fetch_all(pool.as_ref())
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
                    .map(|location| LocationDetails {
                        id: primary_location.location_id.into(),
                        name: location.name.to_owned().into(),
                        address: location.address.to_owned(),
                    })
            })
            .collect_vec();

        Ok(locations)
    }
}
