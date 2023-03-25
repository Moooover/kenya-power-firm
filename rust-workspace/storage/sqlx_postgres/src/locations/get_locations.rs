use sqlx::PgPool;
use std::collections::{HashMap, HashSet};

use crate::repository::Repository;
use anyhow::Context;
use entities::locations::{LocationDetails, LocationId, LocationName};
use itertools::Itertools;

impl Repository {
    pub async fn get_locations_by_ids(
        &self,
        ids: HashSet<LocationId>,
    ) -> anyhow::Result<HashMap<LocationId, LocationDetails>> {
        let ids = ids.into_iter().map(|id| id.inner()).collect_vec();
        let results = sqlx::query!(
            "
            SELECT id, name FROM location.locations WHERE id = ANY($1)
            ",
            &ids[..]
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch locations")?;

        let results = results
            .into_iter()
            .map(|record| {
                (
                    LocationId::from(record.id),
                    LocationDetails {
                        id: LocationId::from(record.id),
                        name: LocationName::from(record.name),
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        Ok(results)
    }
}
