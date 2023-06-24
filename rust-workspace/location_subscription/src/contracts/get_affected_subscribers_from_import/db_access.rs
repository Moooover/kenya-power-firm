use crate::contracts::get_affected_subscribers_from_import::Region;
use crate::data_transfer::{
    AffectedSubscriber, LineWithScheduledInterruptionTime, LocationMatchedAndLineSchedule,
};
use crate::db_access::DbAccess;
use crate::save_and_search_for_locations::{AffectedLocation, SaveAndSearchLocations};
use anyhow::Context;
use entities::subscriptions::SubscriberId;
use itertools::Itertools;
use shared_kernel::location_ids::LocationId;
use std::collections::{HashMap, HashSet};
use std::iter;
use url::Url;

pub struct AffectedSubscribersDbAccess {
    location_search: SaveAndSearchLocations,
    db_access: DbAccess,
}

impl From<(AffectedLocation, String)> for LocationMatchedAndLineSchedule {
    fn from((value, location_name): (AffectedLocation, String)) -> Self {
        Self {
            line_schedule: LineWithScheduledInterruptionTime {
                line_name: value.line_matched.line_name,
                from: value.line_matched.from,
                to: value.line_matched.to,
                source_url: value.line_matched.source_url,
            },
            location_name,
            location_id: value.location_id,
        }
    }
}

impl AffectedSubscribersDbAccess {
    pub fn new() -> Self {
        Self {
            location_search: SaveAndSearchLocations::new(),
            db_access: DbAccess,
        }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn get_affected_subscribers(
        &self,
        url: Url,
        regions: &[Region],
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<LocationMatchedAndLineSchedule>>> {
        let locations_matched = self
            .location_search
            .get_affected_locations_from_regions(url, regions)
            .await?;

        self.affected_subscribers_from_affected_locations(locations_matched)
            .await
    }

    pub async fn affected_subscribers_from_affected_locations(
        &self,
        locations_matched: Vec<AffectedLocation>,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<LocationMatchedAndLineSchedule>>> {
        let location_ids = locations_matched
            .iter()
            .map(|data| data.location_id)
            .collect_vec();
        let subscribers = self
            .subscribers_subscribed_to_locations(&location_ids)
            .await?;

        let mapping_of_location_id_to_affected_locations = locations_matched
            .into_iter()
            .map(|data| (data.location_id, data))
            .collect::<HashMap<_, _>>();

        let result = subscribers
            .into_iter()
            .flat_map(|(subscriber, location_ids)| {
                let (directly_affected, potentially_affected): (Vec<_>, Vec<_>) = location_ids
                    .into_iter()
                    .filter_map(|location_id| {
                        mapping_of_location_id_to_affected_locations
                            .get(&location_id)
                            .cloned()
                    })
                    .partition(|location| location.is_directly_affected);

                iter::once((
                    AffectedSubscriber::DirectlyAffected(subscriber),
                    directly_affected,
                ))
                .into_iter()
                .chain(iter::once((
                    AffectedSubscriber::PotentiallyAffected(subscriber),
                    potentially_affected,
                )))
                .collect::<HashMap<_, _>>()
            })
            .collect::<HashMap<_, _>>();

        let mapping_of_ids_to_names = {
            let location_ids = result
                .values()
                .flat_map(|affected_locations| {
                    affected_locations
                        .iter()
                        .map(|location| location.location_id)
                })
                .collect::<HashSet<_>>();
            self.get_location_name_by_ids(location_ids).await?
        };

        let result = result
            .into_iter()
            .map(|(subscriber, affected_locations)| {
                let locations = affected_locations
                    .into_iter()
                    .filter_map(|location| {
                        mapping_of_ids_to_names
                            .get(&location.location_id)
                            .map(|location_name| (location, location_name.to_owned()))
                    })
                    .collect_vec();
                (
                    subscriber,
                    locations.into_iter().map(Into::into).collect_vec(),
                )
            })
            .collect();

        Ok(result)
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn get_location_name_by_ids(
        &self,
        ids: HashSet<LocationId>,
    ) -> anyhow::Result<HashMap<LocationId, String>> {
        let pool = self.db_access.pool().await;
        let ids = ids.into_iter().map(|id| id.inner()).collect_vec();
        let results = sqlx::query!(
            "
            SELECT id, name FROM location.locations WHERE id = ANY($1)
            ",
            &ids[..]
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to fetch locations")?;

        let results = results
            .into_iter()
            .map(|record| (LocationId::from(record.id), record.name))
            .collect::<HashMap<_, _>>();
        Ok(results)
    }

    async fn subscribers_subscribed_to_locations(
        &self,
        location_ids: &[LocationId],
    ) -> anyhow::Result<HashMap<SubscriberId, Vec<LocationId>>> {
        let pool = self.db_access.pool().await;
        let location_ids = location_ids
            .iter()
            .map(|location| location.inner())
            .collect_vec();
        let records = sqlx::query!(
            "
            SELECT subscriber_id, location_id FROM location.subscriber_locations
            WHERE location_id = ANY($1)
            ",
            &location_ids[..]
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to fetch location.subscriber_locations")?;
        let mapping = records
            .into_iter()
            .map(|data| {
                (
                    SubscriberId::from(data.subscriber_id),
                    LocationId::from(data.location_id),
                )
            })
            .into_group_map();
        Ok(mapping)
    }
}
