pub mod search_engine;
mod searcheable_candidate;

use crate::contracts::get_affected_subscribers_from_import::{
    Area, Region, TimeFrame as ContractTimeFrame,
};
use crate::data_transfer::LineWithScheduledInterruptionTime;
use crate::db_access::DbAccess;
use crate::save_and_search_for_locations::searcheable_candidate::NonAcronymString;
use anyhow::{anyhow, Context};
use entities::locations::{ExternalLocationId, LocationId};
use entities::power_interruptions::location::{AreaName, FutureOrCurrentNairobiTZDateTime};
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use searcheable_candidate::SearcheableCandidates;
use serde::Deserialize;
use shared_kernel::date_time::nairobi_date_time::NairobiTZDateTime;
use shared_kernel::date_time::time_frame::TimeFrame;
use shared_kernel::uuid_key;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use std::collections::HashMap;
use tracing::error;
use url::Url;
use uuid::Uuid;

uuid_key!(NearbyLocationId);

pub struct SaveAndSearchLocations {
    db_access: DbAccess,
}

#[derive(Clone, Debug)]
pub struct LocationInput {
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct AffectedLocation {
    pub location_id: LocationId,
    pub line_matched: LineWithScheduledInterruptionTime,
    pub is_directly_affected: bool,
}

pub struct LocationWithCoordinates {
    pub location_id: LocationId,
    pub latitude: f64,
    pub longitude: f64,
}

impl Default for SaveAndSearchLocations {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveAndSearchLocations {
    pub fn new() -> Self {
        Self {
            db_access: DbAccess,
        }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn save_main_location(&self, location: LocationInput) -> anyhow::Result<LocationId> {
        let pool = self.db_access.pool().await;
        let external_id = location.external_id.as_ref();
        let address = location.address.clone();
        let sanitized_address = NonAcronymString::from(location.address);

        sqlx::query!(
            "
            INSERT INTO location.locations (name, external_id, address, sanitized_address, external_api_response) 
            VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING
            ",
            &location.name,
            external_id,
            address,
            sanitized_address.as_ref(),
            Json(location.api_response.clone()) as _
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to insert location")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.locations WHERE external_id = $1
            "#,
            external_id
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to get inserted location")?;
        let id = record.id.into();
        search_engine::save_primary_location::execute(
            id,
            location.name,
            external_id.into(),
            sanitized_address.to_string(),
            location.api_response,
        )
        .await?;
        Ok(id)
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn affected_location(
        &self,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let directly_affected =
            directly_affected_location::execute(&self.db_access, location_id).await?;

        if directly_affected.is_some() {
            return Ok(directly_affected);
        }
        potentially_affected_location::execute(&self.db_access, location_id).await
    }

    pub async fn find_location_coordinates_by_external_id(
        &self,
        location: ExternalLocationId,
    ) -> anyhow::Result<Option<LocationWithCoordinates>> {
        #[derive(Deserialize)]

        struct LatitudeAndLongitude {
            lat: f64,
            lng: f64,
        }

        #[derive(Deserialize)]

        struct Geometry {
            location: LatitudeAndLongitude,
        }

        #[derive(Deserialize)]
        struct DataResult {
            geometry: Geometry,
        }

        #[derive(Deserialize)]
        struct ResultWrapper {
            result: DataResult,
        }

        #[derive(Deserialize)]
        struct Row {
            id: Uuid,
            value: Json<ResultWrapper>,
        }
        let pool = self.db_access.pool().await;
        let result = sqlx::query_as!(
            Row,
            r#"
            SELECT id, external_api_response as "value: Json<ResultWrapper>" FROM location.locations WHERE external_id = $1
            "#,
            location.inner()
        )
        .fetch_optional(pool.as_ref())
        .await.with_context(|| {
            format!("Failed to get response from FROM location.locations WHERE external_id = {}", location.inner())
        })?;

        let result = result.map(|data| LocationWithCoordinates {
            location_id: data.id.into(),
            latitude: data.value.result.geometry.location.lat,
            longitude: data.value.result.geometry.location.lng,
        });

        Ok(result)
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn get_affected_locations_from_regions(
        &self,
        url: Url,
        regions: &[Region],
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        let areas = regions
            .iter()
            .flat_map(|region| {
                region
                    .counties
                    .iter()
                    .flat_map(|county| &county.areas)
                    .collect_vec()
            })
            .collect_vec();
        let mut futures: FuturesUnordered<_> = areas
            .into_iter()
            .map(|area| affected_locations_in_an_area::execute(area, url.clone(), &self.db_access))
            .collect();

        let mut result = vec![];

        while let Some(future_result) = futures.next().await {
            match future_result {
                Ok(area_results) => {
                    result.push(area_results);
                }
                Err(e) => {
                    error!("Error searching locations {e:?}", e = e)
                }
            }
        }
        Ok(result.into_iter().flatten().collect_vec())
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn was_nearby_location_already_saved(
        &self,
        location_id: LocationId,
    ) -> anyhow::Result<Option<NearbyLocationId>> {
        let pool = self.db_access.pool().await;
        let location_id = location_id.inner();
        let db_results = sqlx::query!(
            r#"
            SELECT id
            FROM location.nearby_locations WHERE location_id = $1
            "#,
            location_id
        )
        .fetch_optional(pool.as_ref())
        .await
        .context("Failed to fetch nearby_locations")?;

        Ok(db_results.map(|record| record.id.into()))
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub(super) async fn save_nearby_locations(
        &self,
        url: Url,
        primary_location: LocationId,
        api_response: serde_json::Value,
    ) -> anyhow::Result<NearbyLocationId> {
        let pool = self.db_access.pool().await;
        sqlx::query!(
            "
            INSERT INTO location.nearby_locations (source_url, location_id, response) 
            VALUES ($1, $2, $3) ON CONFLICT DO NOTHING
            ",
            url.to_string(),
            primary_location.inner(),
            Json(api_response.clone()) as _
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to insert nearby_locations")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.nearby_locations WHERE source_url = $1
            "#,
            url.to_string()
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch the nearby_location by id")?;
        let nearby_location_id = record.id.into();
        search_engine::save_nearby_location::execute(
            primary_location,
            api_response,
            nearby_location_id,
        )
        .await?;
        Ok(nearby_location_id)
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn currently_affected_locations(&self) -> anyhow::Result<Vec<AffectedLocation>> {
        let bare_results = BareAffectedLine::lines_affected_in_the_future(&self.db_access).await?;
        let mut results = vec![];
        for (area_name, lines) in bare_results.iter() {
            let first_line = lines.first();
            if let Some(first_line) = first_line {
                let url = first_line.url.clone();
                let area = Area {
                    name: area_name.to_string(),
                    time_frame: ContractTimeFrame {
                        from: FutureOrCurrentNairobiTZDateTime::try_from(first_line.time_frame.from.clone())
                            .map_err(|err| anyhow!(err))
                            .with_context(|| format!("Received Date time that passed: {:?} yet the time should have been in the future", first_line.time_frame.from))?,
                        to: FutureOrCurrentNairobiTZDateTime::try_from(first_line.time_frame.to.clone())
                            .map_err(|err| anyhow!(err))
                            .with_context(|| format!("Received Date time that passed: {:?} yet the time should have been in the future", first_line.time_frame.from))?,
                    },
                    locations: lines.iter().map(|line| line.line.clone()).collect_vec(),
                };
                results.extend(
                    affected_locations_in_an_area::execute(&area, url, &self.db_access).await?,
                );
            }
        }
        Ok(results)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BareAffectedLine {
    pub line: String,
    pub url: Url,
    pub time_frame: TimeFrame<NairobiTZDateTime>,
}

impl BareAffectedLine {
    pub(crate) async fn lines_affected_in_the_future(
        db: &DbAccess,
    ) -> anyhow::Result<HashMap<AreaName, Vec<Self>>> {
        #[derive(sqlx::FromRow, Debug)]
        struct DbAreaLine {
            line_name: String,
            area_name: String,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
            url: String,
        }
        let pool = db.pool().await;
        let results = sqlx::query_as::<_, DbAreaLine>(
            "
                WITH upcoming_scheduled_blackouts AS (
                  SELECT schedule.id, url, start_time, end_time FROM location.blackout_schedule schedule INNER JOIN  source ON  schedule.source_id = source.id WHERE end_time > now()
                ), blackout_schedule_with_lines_and_areas AS (
                  SELECT line_id, url, start_time, end_time, name, area_id FROM location.line_schedule INNER JOIN location.line ON line_schedule.line_id = location.line.id INNER JOIN upcoming_scheduled_blackouts ON line_schedule.schedule_id = upcoming_scheduled_blackouts.id
                ),line_area_source_url AS (
                  SELECT blackout_schedule_with_lines_and_areas.name as line_name, location.area.name as area_name, start_time, end_time , url FROM blackout_schedule_with_lines_and_areas INNER JOIN location.area ON blackout_schedule_with_lines_and_areas.area_id = location.area.id
                )
                SELECT * FROM line_area_source_url
                "
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to get lines that will be affected")?;

        let results = results
            .into_iter()
            .map(|data| {
                let url_result = Url::parse(&data.url);
                url_result.map(|url| {
                    (
                        data.area_name.into(),
                        BareAffectedLine {
                            line: data.line_name,
                            url,
                            time_frame: TimeFrame {
                                from: NairobiTZDateTime::from(data.start_time),
                                to: NairobiTZDateTime::from(data.end_time),
                            },
                        },
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to map urls")?;

        Ok(results.into_iter().into_group_map())
    }
}

struct BareAffectedLinesMapping<'a> {
    mapping_of_line_to_time_frame: HashMap<&'a String, &'a TimeFrame<NairobiTZDateTime>>,
    mapping_of_searcheble_candidate_to_original_line_candidate: HashMap<String, &'a String>,
    mapping_of_line_to_url: HashMap<&'a String, &'a Url>,
}

impl<'a> BareAffectedLinesMapping<'a> {
    fn generate(affected_lines: &'a [BareAffectedLine]) -> Self {
        let (mapping_of_line_to_time_frame, mapping_of_line_to_url): (
            HashMap<_, _>,
            HashMap<_, _>,
        ) = affected_lines
            .iter()
            .map(|line| ((&line.line, &line.time_frame), (&line.line, &line.url)))
            .unzip();
        let mapping_of_searcheble_candidate_to_original_line_candidate = affected_lines
            .iter()
            .flat_map(|affected_line| {
                let candidates = SearcheableCandidates::from(affected_line.line.as_ref());
                candidates
                    .into_inner()
                    .into_iter()
                    .map(|candidate| (candidate, &affected_line.line))
            })
            .collect::<HashMap<_, _>>();
        Self {
            mapping_of_line_to_time_frame,
            mapping_of_searcheble_candidate_to_original_line_candidate,
            mapping_of_line_to_url,
        }
    }
}

struct AffectedLocationGenerator<'a> {
    affected_lines: &'a [BareAffectedLine],
}

impl<'a> AffectedLocationGenerator<'a> {
    fn generate(
        &self,
        search_query: String,
        location_id: LocationId,
        is_directly_affected: bool,
    ) -> anyhow::Result<AffectedLocation> {
        let BareAffectedLinesMapping {
            mapping_of_line_to_time_frame,
            mapping_of_searcheble_candidate_to_original_line_candidate,
            mapping_of_line_to_url,
        } = BareAffectedLinesMapping::generate(self.affected_lines);
        let original_line_candidate = mapping_of_searcheble_candidate_to_original_line_candidate
            .get(&search_query)
            .ok_or(anyhow!(
                "Failed to get original_line_candidate from search_query {}",
                search_query
            ))?;

        let time_frame = *mapping_of_line_to_time_frame
            .get(original_line_candidate)
            .ok_or(anyhow!(
            "Failed to get time_frame when we should have for candidate {original_line_candidate}"
        ))?;

        let url = *mapping_of_line_to_url
            .get(original_line_candidate)
            .ok_or(anyhow!(
                "Failed to get url from mapping_of_line_to_url for line {original_line_candidate}"
            ))?;

        Ok(AffectedLocation {
            location_id,
            line_matched: LineWithScheduledInterruptionTime {
                line_name: original_line_candidate.to_string(),
                from: time_frame.from.clone(),
                to: time_frame.to.clone(),
                source_url: url.clone(),
            },
            is_directly_affected,
        })
    }
}

#[derive(sqlx::FromRow, Debug)]
struct DbLocationSearchResults {
    pub search_query: String,
    #[allow(dead_code)]
    pub location: String,
    #[allow(dead_code)]
    pub id: Uuid,
}

mod directly_affected_location {
    use std::collections::HashMap;

    use crate::db_access::DbAccess;
    use crate::save_and_search_for_locations::{
        AffectedLocation, AffectedLocationGenerator, BareAffectedLine, DbLocationSearchResults,
    };
    use anyhow::Context;
    use entities::locations::LocationId;
    use entities::power_interruptions::location::AreaName;
    use itertools::Itertools;

    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableCandidates;

    use super::search_engine;
    use anyhow::anyhow;

    #[tracing::instrument(err, skip(db), level = "info")]
    pub(super) async fn execute(
        db: &DbAccess,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let results = BareAffectedLine::lines_affected_in_the_future(db).await?;
        for (area_name, affected_lines) in results.iter() {
            let affected_location =
                directly_affected_location(db, location_id, area_name, affected_lines).await?;
            if let Some(affected_location) = affected_location {
                return Ok(Some(affected_location));
            }
        }

        Ok(None)
    }

    #[tracing::instrument(err, skip(db), level = "info")]
    async fn directly_affected_location(
        db: &DbAccess,
        location_id: LocationId,
        area_name: &AreaName,
        affected_lines: &[BareAffectedLine],
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let searcheable_candidates = affected_lines
            .iter()
            .flat_map(|line| SearcheableCandidates::from(line.line.as_ref()).into_inner())
            .collect_vec();
        let primary_location = get_primary_location_search_result(
            location_id,
            area_name,
            db,
            searcheable_candidates.clone(),
        )
        .await?;

        if let Some(location) = primary_location {
            let affected_location = AffectedLocationGenerator { affected_lines }.generate(
                location.search_query,
                location_id,
                true,
            )?;
            return Ok(Some(affected_location));
        }

        let search_engine = search_engine::directly_affected_area_locations::DirectlyAffectedLocationsSearchEngine::new(area_name.clone());
        let mapping_of_original_candidate_to_searcheable_candidate = searcheable_candidates
            .into_iter()
            .map(|candidate| (candidate.clone(), format!("{candidate} {location_id}")))
            .collect::<HashMap<_, _>>();

        let searcheable_candidates = mapping_of_original_candidate_to_searcheable_candidate
            .values()
            .cloned()
            .collect_vec();

        let results = search_engine.search(searcheable_candidates).await?;
        if let Some((location_id, search_query)) = results.into_iter().next() {
            let original_candidate = mapping_of_original_candidate_to_searcheable_candidate
                .get(&search_query)
                .ok_or_else(|| {
                    anyhow!(
                        "Failed to find original candidate of search_query = {} from mapping {:?}",
                        &search_query,
                        &mapping_of_original_candidate_to_searcheable_candidate
                    )
                })
                .cloned()?;
            let affected_location = AffectedLocationGenerator { affected_lines }.generate(
                original_candidate,
                location_id,
                true,
            )?;
            return Ok(Some(affected_location));
        }
        Ok(None)
    }
    #[tracing::instrument(err, skip(db), level = "info")]
    async fn get_primary_location_search_result(
        location_id: LocationId,
        area_name: &AreaName,
        db: &DbAccess,
        searcheable_candidates: Vec<String>,
    ) -> anyhow::Result<Option<DbLocationSearchResults>> {
        let mut primary_location: Option<DbLocationSearchResults> = None;
        let pool = db.pool().await;
        for (searcheable_candidates, location_id, searcheable_area) in
            SearcheableCandidates::from_area_name(area_name)
                .into_iter()
                .flat_map(|area_candidate| {
                    area_candidate
                        .into_inner()
                        .into_iter()
                        .map(|area_candidate| {
                            (
                                searcheable_candidates.clone(),
                                location_id.inner(),
                                area_candidate,
                            )
                        })
                })
        {
            let location = sqlx::query_as::<_, DbLocationSearchResults>(
                "
                    SELECT * FROM location.search_specific_location_primary_text($1::text[], $2::uuid, $3::text)
                    ",
            )
                .bind(searcheable_candidates)
                .bind(location_id)
                .bind(searcheable_area.to_string())
                .fetch_optional(pool.as_ref())
                .await
                .context("Failed to fetch results from search_specific_location_primary_text")?;
            if let Some(location) = location {
                primary_location = Some(location);
                break;
            }
        }
        Ok(primary_location)
    }
}

mod potentially_affected_location {
    use std::collections::HashMap;

    use super::search_engine;
    use crate::db_access::DbAccess;
    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableCandidates;
    use crate::save_and_search_for_locations::{
        AffectedLocation, AffectedLocationGenerator, BareAffectedLine, DbLocationSearchResults,
    };
    use anyhow::anyhow;
    use anyhow::Context;
    use entities::locations::LocationId;
    use entities::power_interruptions::location::AreaName;
    use itertools::Itertools;

    #[tracing::instrument(err, skip(db), level = "info")]
    pub async fn execute(
        db: &DbAccess,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let results = BareAffectedLine::lines_affected_in_the_future(db).await?;

        for (area_name, affected_lines) in results.iter() {
            let affected_location =
                potentially_affected_location(db, location_id, area_name, affected_lines).await?;
            if let Some(affected_location) = affected_location {
                return Ok(Some(affected_location));
            }
        }

        Ok(None)
    }

    #[tracing::instrument(err, skip(db), level = "info")]
    async fn potentially_affected_location(
        db: &DbAccess,
        location_id: LocationId,
        area_name: &AreaName,
        affected_lines: &[BareAffectedLine],
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let searcheable_candidates = affected_lines
            .iter()
            .flat_map(|line| SearcheableCandidates::from(line.line.as_ref()).into_inner())
            .collect_vec();
        let potentially_affected_location = get_potentially_affected_location_search_result(
            location_id,
            area_name,
            db,
            searcheable_candidates.clone(),
        )
        .await?;

        if let Some(location) = potentially_affected_location {
            let affected_location = AffectedLocationGenerator { affected_lines }.generate(
                location.search_query,
                location_id,
                true,
            )?;
            return Ok(Some(affected_location));
        }

        let search_engine =
            search_engine::potentially_affected_area_locations::NearbyLocationsSearchEngine::new(
                area_name.clone(),
            );

        let mapping_of_original_candidate_to_searcheable_candidate = searcheable_candidates
            .into_iter()
            .map(|candidate| (candidate.clone(), format!("{candidate} {location_id}")))
            .collect::<HashMap<_, _>>();

        let searcheable_candidates = mapping_of_original_candidate_to_searcheable_candidate
            .values()
            .cloned()
            .collect_vec();

        let results = search_engine.search(searcheable_candidates).await?;
        if let Some((location_id, query)) = results.into_iter().next() {
            let original_candidate = mapping_of_original_candidate_to_searcheable_candidate
                .get(&query)
                .ok_or_else(|| {
                    anyhow!(
                        "Failed to find original candidate of search_query = {} from mapping {:?}",
                        &query,
                        &mapping_of_original_candidate_to_searcheable_candidate
                    )
                })
                .cloned()?;
            let affected_location = AffectedLocationGenerator { affected_lines }.generate(
                original_candidate,
                location_id,
                true,
            )?;
            return Ok(Some(affected_location));
        }

        Ok(None)
    }

    async fn get_potentially_affected_location_search_result(
        location_id: LocationId,
        area_name: &AreaName,
        db: &DbAccess,
        searcheable_candidates: Vec<String>,
    ) -> anyhow::Result<Option<DbLocationSearchResults>> {
        let mut result: Option<DbLocationSearchResults> = None;
        let pool = db.pool().await;
        for (searcheable_candidates, location_id, searcheable_area) in
            SearcheableCandidates::from_area_name(area_name)
                .into_iter()
                .flat_map(|area_candidate| {
                    area_candidate
                        .into_inner()
                        .into_iter()
                        .map(|area_candidate| {
                            (
                                searcheable_candidates.clone(),
                                location_id.inner(),
                                area_candidate,
                            )
                        })
                })
        {
            let location = sqlx::query_as::<_, DbLocationSearchResults>(
                "
                    SELECT * FROM location.search_nearby_locations_with_nearby_location_id_and_area_name($1::text[], $2::uuid, $3::text)
                    ",
            )
                .bind(searcheable_candidates)
                .bind(location_id)
                .bind(searcheable_area.to_string())
                .fetch_optional(pool.as_ref())
                .await
                .context("Failed to fetch results from search_nearby_locations_with_nearby_location_id_and_area_name")?;
            if let Some(location) = location {
                result = Some(location);
                break;
            }
        }
        Ok(result)
    }
}

mod affected_locations_in_an_area {
    use crate::contracts::get_affected_subscribers_from_import::{Area, TimeFrame};
    use crate::data_transfer::LineWithScheduledInterruptionTime;
    use crate::db_access::DbAccess;
    use crate::save_and_search_for_locations::AffectedLocation;
    use anyhow::Context;
    use entities::power_interruptions::location::AreaName;
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;
    use itertools::Itertools;
    use std::collections::{HashMap, HashSet};
    use url::Url;
    use uuid::Uuid;

    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableCandidates;

    use super::search_engine::{
        self, directly_affected_area_locations::DirectlyAffectedLocationsSearchEngine,
        potentially_affected_area_locations::NearbyLocationsSearchEngine,
    };

    #[derive(sqlx::FromRow)]
    struct DbLocationSearchResults {
        pub search_query: String,
        #[allow(dead_code)]
        pub location: String,
        pub id: Uuid,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct AffectedLocationKey {
        location_id: Uuid,
        source_url: Url,
        line_name: String,
    }

    impl AffectedLocationKey {
        fn new(affected_location: &AffectedLocation) -> AffectedLocationKey {
            AffectedLocationKey {
                location_id: affected_location.location_id.inner(),
                source_url: affected_location.line_matched.source_url.clone(),
                line_name: affected_location.line_matched.line_name.clone(),
            }
        }
    }

    #[tracing::instrument(err, skip(db), level = "info")]
    pub async fn execute(
        area: &Area,
        source_url: Url,
        db: &DbAccess,
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        let candidates = &area.locations;
        let time_frame = area.time_frame.clone();

        let mapping_of_searcheable_location_candidate_to_candidate = candidates
            .iter()
            .map(|candidate| {
                (
                    SearcheableCandidates::from(candidate.as_ref()),
                    candidate.as_str(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_location_candidate_to_candidate_copy =
            mapping_of_searcheable_location_candidate_to_candidate.clone();

        let searcheable_candidates = mapping_of_searcheable_location_candidate_to_candidate
            .keys()
            .flat_map(|candidate| candidate.inner())
            .collect_vec();

        let area_name = AreaName::new(area.name.clone());
        let searcheable_area_names = SearcheableCandidates::from_area_name(&area_name);

        let mapping_of_searcheable_candidate_to_original_candidate =
            mapping_of_searcheable_location_candidate_to_candidate_copy
                .into_iter()
                .chain(
                    searcheable_area_names
                        .iter()
                        .map(|searcheable_area| (searcheable_area.clone(), area.name.as_str())),
                )
                .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_candidate_to_candidate =
            mapping_of_searcheable_candidate_to_original_candidate
                .iter()
                .flat_map(|(searcheable_candidates, original_candidate)| {
                    searcheable_candidates
                        .inner()
                        .into_iter()
                        .map(|candidate| (candidate, original_candidate.to_string()))
                })
                .collect::<HashMap<_, _>>();

        let directly_affected_search_engine =
            search_engine::directly_affected_area_locations::DirectlyAffectedLocationsSearchEngine::new(area_name.clone());

        let directly_affected_locations = directly_affected_locations(
            db,
            &searcheable_area_names,
            &searcheable_candidates,
            time_frame.clone(),
            &mapping_of_searcheable_candidate_to_candidate,
            source_url.clone(),
            directly_affected_search_engine,
        )
        .await?;

        let nearby_area_locations_search_engine =
            search_engine::potentially_affected_area_locations::NearbyLocationsSearchEngine::new(
                area_name,
            );

        let potentially_affected_locations = potentially_affected_locations(
            db,
            &searcheable_area_names,
            &searcheable_candidates,
            time_frame.clone(),
            &mapping_of_searcheable_candidate_to_candidate,
            source_url,
            nearby_area_locations_search_engine,
        )
        .await?;

        Ok(filter_out_duplicate_affected_locations(
            directly_affected_locations,
            potentially_affected_locations,
        ))
    }

    #[tracing::instrument(err, skip(db, search_engine), level = "info")]
    async fn directly_affected_locations(
        db: &DbAccess,
        searcheable_area_names: &[SearcheableCandidates],
        searcheable_candidates: &[String],
        time_frame: TimeFrame,
        mapping_of_searcheable_candidate_to_candidate: &HashMap<String, String>,
        source: Url,
        search_engine: DirectlyAffectedLocationsSearchEngine,
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        let pool = db.pool().await;

        let searcheable_area_names = searcheable_area_names
            .iter()
            .flat_map(|area_name| area_name.inner().into_iter())
            .collect_vec();

        let mut futures: FuturesUnordered<_> = searcheable_area_names
            .iter()
            .map(|area_name| {
                sqlx::query_as::<_, DbLocationSearchResults>(
                    "
                        SELECT * FROM location.search_locations_primary_text($1::text[], $2::text)
                        ",
                )
                .bind(searcheable_candidates)
                .bind(area_name)
                .fetch_all(pool.as_ref())
            })
            .collect();
        let mut primary_locations = vec![];
        while let Some(result) = futures.next().await {
            primary_locations.push(result.context("Failed to get primary search results from db")?);
        }
        let primary_locations = primary_locations.into_iter().flatten().collect_vec();

        let candidates_not_found = searcheable_candidates
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
            .difference(
                &primary_locations
                    .iter()
                    .map(|location| location.search_query.clone())
                    .collect(),
            )
            .filter_map(|candidate| {
                mapping_of_searcheable_candidate_to_candidate
                    .get(candidate)
                    .cloned()
            })
            .collect_vec();

        let locations_from_candidates_not_found =
            search_engine.search(candidates_not_found).await?;

        let results = primary_locations
            .into_iter()
            .map(|data| (data.search_query, data.id.into()))
            .chain(
                locations_from_candidates_not_found
                    .into_iter()
                    .map(|(location_id, query)| (query, location_id)),
            )
            .filter_map(|(search_query, location_id)| {
                mapping_of_searcheable_candidate_to_candidate
                    .get(&search_query)
                    .map(|original_candidate| AffectedLocation {
                        location_id,
                        line_matched: LineWithScheduledInterruptionTime {
                            line_name: original_candidate.to_string(),
                            from: time_frame.from.as_ref().clone(),
                            to: time_frame.to.as_ref().clone(),
                            source_url: source.clone(),
                        },
                        is_directly_affected: false,
                    })
            })
            .collect_vec();
        Ok(results)
    }

    #[tracing::instrument(err, skip(db, nearby_area_locations_search_engine), level = "info")]
    async fn potentially_affected_locations(
        db: &DbAccess,
        searcheable_area_names: &[SearcheableCandidates],
        searcheable_candidates: &[String],
        time_frame: TimeFrame,
        mapping_of_searcheable_candidate_to_candidate: &HashMap<String, String>,
        source: Url,
        nearby_area_locations_search_engine: NearbyLocationsSearchEngine,
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        let searcheable_area_names = searcheable_area_names
            .iter()
            .flat_map(|area_name| area_name.inner().into_iter())
            .collect_vec();
        #[derive(sqlx::FromRow, Debug)]
        pub struct NearbySearchResult {
            candidate: String,
            location_id: Uuid,
        }
        let pool = db.pool().await;

        let mut futures: FuturesUnordered<_> = searcheable_area_names
            .iter()
            .map(|area_name| {
                sqlx::query_as::<_, NearbySearchResult>(
                    "
                        SELECT * FROM location.search_nearby_locations_with_area_name($1::text[], $2::text)
                        ",
                )
                .bind(searcheable_candidates)
                .bind(area_name)
                .fetch_all(pool.as_ref())
            })
            .collect();
        let mut nearby_locations = vec![];
        while let Some(result) = futures.next().await {
            nearby_locations
                .push(result.context("Failed to get nearby_locations results  from db")?);
        }

        let nearby_locations = nearby_locations.into_iter().flatten().collect_vec();

        let candidates_not_found = searcheable_candidates
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
            .difference(
                &nearby_locations
                    .iter()
                    .map(|location| location.candidate.clone())
                    .collect(),
            )
            .filter_map(|candidate| {
                mapping_of_searcheable_candidate_to_candidate
                    .get(candidate)
                    .cloned()
            })
            .collect_vec();

        let search_engine_nearby_locations_results = nearby_area_locations_search_engine
            .search(candidates_not_found)
            .await?;

        let location_ids_to_search_query = nearby_locations
            .iter()
            .map(|data| (data.location_id, data.candidate.clone()))
            .chain(
                search_engine_nearby_locations_results
                    .iter()
                    .map(|(location_id, candidate)| (location_id.inner(), candidate.to_owned())),
            )
            .collect::<HashMap<_, _>>();

        let results = nearby_locations
            .iter()
            .map(|data| data.location_id)
            .chain(
                search_engine_nearby_locations_results
                    .keys()
                    .map(|id| id.inner()),
            )
            .filter_map(|location_id| {
                location_ids_to_search_query
                    .get(&location_id)
                    .and_then(|candidate| {
                        mapping_of_searcheable_candidate_to_candidate
                            .get(candidate)
                            .cloned()
                            .map(|line| (line, location_id))
                    })
                    .map(|(line, location)| AffectedLocation {
                        location_id: location.into(),
                        line_matched: LineWithScheduledInterruptionTime {
                            line_name: line,
                            from: time_frame.from.as_ref().clone(),
                            to: time_frame.to.as_ref().clone(),
                            source_url: source.clone(),
                        },
                        is_directly_affected: false,
                    })
            })
            .collect_vec();

        Ok(results)
    }

    fn filter_out_duplicate_affected_locations(
        directly_affected_locations: Vec<AffectedLocation>,
        potentially_affected_locations: Vec<AffectedLocation>,
    ) -> Vec<AffectedLocation> {
        let directly_affected_location_keys = directly_affected_locations
            .iter()
            .map(AffectedLocationKey::new)
            .collect::<HashSet<_>>();
        let potentially_affected_locations = potentially_affected_locations
            .into_iter()
            .filter(|location| {
                !directly_affected_location_keys.contains(&AffectedLocationKey::new(location))
            })
            .collect_vec();

        directly_affected_locations
            .into_iter()
            .chain(potentially_affected_locations.into_iter())
            .collect_vec()
    }
}
