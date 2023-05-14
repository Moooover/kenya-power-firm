use crate::data_transfer::LineWithScheduledInterruptionTime;
use crate::db_access::DbAccess;
use anyhow::{anyhow, Context};
use entities::locations::{ExternalLocationId, LocationId};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex, RegexBuilder};
use sqlx::types::Json;
use std::collections::HashMap;

use crate::contracts::get_affected_subscribers_from_import::Region;
use entities::power_interruptions::location::{AreaName, NairobiTZDateTime, TimeFrame};
use futures::{stream::FuturesUnordered, StreamExt};
use serde::Deserialize;
use shared_kernel::uuid_key;
use sqlx::types::chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};
use url::Url;
use uuid::Uuid;

uuid_key!(NearbyLocationId);

lazy_static! {
    static ref ACRONYM_MAP: HashMap<String, &'static str> = HashMap::from([
        ("pri".to_string(), "Primary"),
        ("rd".to_string(), "Road"),
        ("est".to_string(), "Estate"),
        ("sch".to_string(), "School"),
        ("sec".to_string(), "Secondary"),
        ("stn".to_string(), "Station"),
        ("apts".to_string(), "Apartments"),
        ("hqtrs".to_string(), "Headquaters"),
        ("mkt".to_string(), "Market"),
        ("fact".to_string(), "Factory"),
        ("t/fact".to_string(), "Tea Factory"),
        ("c/fact".to_string(), "Coffee Factory")
    ]);
    static ref REGEX_STR: String = {
        let keys = ACRONYM_MAP.keys().join("|");
        format!(r"\b(?:{})\b", keys)
    };
    static ref ACRONYMS_MATCHER: Regex = RegexBuilder::new(&REGEX_STR)
        .case_insensitive(true)
        .build()
        .expect("ACRONYMS_MATCHER to have been built successfully");
}

#[derive(Debug)]
pub struct NonAcronymString(String);

impl From<String> for NonAcronymString {
    fn from(value: String) -> Self {
        let result = ACRONYMS_MATCHER
            .replace_all(&value, |cap: &Captures| {
                let cap_as_lower_case = cap[0].to_lowercase();
                ACRONYM_MAP
                    .get(&cap_as_lower_case)
                    .cloned()
                    .unwrap_or_default()
                    .to_string()
            })
            .trim()
            .to_string();

        NonAcronymString(result)
    }
}

impl Display for NonAcronymString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for NonAcronymString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SearcheableCandidate(String);

impl ToString for SearcheableCandidate {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl AsRef<str> for SearcheableCandidate {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SearcheableCandidate {
    pub fn from_area_name(area: &AreaName) -> Vec<Self> {
        area.as_ref()
            .split(',')
            .map(SearcheableCandidate::from)
            .collect_vec()
    }

    // pub fn original_value(&self) -> String {
    //     self.0.replace(" <-> ", " ")
    // }
}

impl From<&str> for SearcheableCandidate {
    fn from(value: &str) -> Self {
        let value = value.trim().replace(' ', " <-> ");
        SearcheableCandidate(value)
    }
}

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
            location.name,
            external_id,
            address,
            sanitized_address.as_ref(),
            Json(location.api_response) as _
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

        Ok(record.id.into())
    }

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
                    // TODO: Refactor to tracing block
                    println!("Error searching locations {e:?}");
                }
            }
        }
        Ok(result.into_iter().flatten().collect_vec())
    }

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
            Json(api_response) as _
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

        Ok(record.id.into())
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
                  SELECT schedule.id, url, start_time, end_time FROM location.blackout_schedule schedule INNER JOIN  source ON  schedule.source_id = source.id WHERE start_time > now()
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
        let (
            mapping_of_line_to_time_frame,
            mapping_of_line_to_url,
            mapping_of_searcheble_candidate_to_original_line_candidate,
        ): (HashMap<_, _>, HashMap<_, _>, HashMap<_, _>) = affected_lines
            .iter()
            .map(|line| {
                (
                    (&line.line, &line.time_frame),
                    (&line.line, &line.url),
                    (
                        SearcheableCandidate::from(line.line.as_ref()).to_string(),
                        &line.line,
                    ),
                )
            })
            .multiunzip();
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

mod directly_affected_location {
    use crate::db_access::DbAccess;
    use crate::save_and_search_for_locations::{
        AffectedLocation, AffectedLocationGenerator, BareAffectedLine,
    };
    use anyhow::Context;
    use entities::locations::LocationId;
    use entities::power_interruptions::location::AreaName;
    use itertools::Itertools;
    use uuid::Uuid;

    use super::SearcheableCandidate;

    #[derive(sqlx::FromRow, Debug)]
    struct DbLocationSearchResults {
        pub search_query: String,
        #[allow(dead_code)]
        pub location: String,
        #[allow(dead_code)]
        pub id: Uuid,
    }
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

    async fn directly_affected_location(
        db: &DbAccess,
        location_id: LocationId,
        area_name: &AreaName,
        affected_lines: &[BareAffectedLine],
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let searcheable_candidates = affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()).to_string())
            .collect_vec();
        let primary_location =
            get_primary_location_search_result(location_id, area_name, db, searcheable_candidates)
                .await?;

        println!("{primary_location:?}");

        if let Some(location) = primary_location {
            let affected_location = AffectedLocationGenerator { affected_lines }.generate(
                location.search_query,
                location_id,
                true,
            )?;
            return Ok(Some(affected_location));
        }

        Ok(None)
    }
    async fn get_primary_location_search_result(
        location_id: LocationId,
        area_name: &AreaName,
        db: &DbAccess,
        searcheable_candidates: Vec<String>,
    ) -> anyhow::Result<Option<DbLocationSearchResults>> {
        let mut primary_location: Option<DbLocationSearchResults> = None;
        let pool = db.pool().await;
        for (searcheable_candidates, location_id, searcheable_area) in
            SearcheableCandidate::from_area_name(area_name)
                .into_iter()
                .map(|area_candidate| {
                    (
                        searcheable_candidates.clone(),
                        location_id.inner(),
                        area_candidate,
                    )
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
    use crate::save_and_search_for_locations::{
        AffectedLocation, AffectedLocationGenerator, BareAffectedLine, NearbyLocationId,
    };
    use crate::{db_access::DbAccess, save_and_search_for_locations::SearcheableCandidate};
    use anyhow::Context;
    use entities::locations::LocationId;
    use itertools::Itertools;
    use std::iter;
    use uuid::Uuid;

    pub async fn execute(
        db: &DbAccess,
        location_id: LocationId,
    ) -> anyhow::Result<Option<AffectedLocation>> {
        let results = BareAffectedLine::lines_affected_in_the_future(db).await?;

        let bare_affected_lines = results
            .into_iter()
            .filter_map(|(area, affected_lines)| {
                affected_lines.first().cloned().map(|line| {
                    iter::once(BareAffectedLine {
                        line: area.to_string(),
                        url: line.url.clone(),
                        time_frame: line.time_frame,
                    })
                    .chain(affected_lines.into_iter())
                })
            })
            .flatten()
            .collect_vec();

        let searcheable_candidates = bare_affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()))
            .map(|candidate| candidate.to_string())
            .collect_vec();

        #[derive(sqlx::FromRow, Debug)]
        struct NearbySearchResult {
            candidate: String,
            location_id: Uuid,
        }

        let pool = db.pool().await;

        let location_id = location_id.inner();

        let nearby_location_id = sqlx::query!(
            "
               SELECT id FROM location.nearby_locations WHERE location_id = $1::uuid
            ",
            location_id
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch nearby_location_id")?;

        let nearby_location_id = NearbyLocationId::from(nearby_location_id.id);

        let nearby_location = sqlx::query_as::<_, NearbySearchResult>(
            "
            SELECT * FROM location.search_nearby_locations_with_nearby_location_id($1::text[], $2::uuid)
            ",
        )
        .bind(searcheable_candidates.clone())
        .bind(nearby_location_id.inner())
        .fetch_optional(pool.as_ref())
        .await.context("Failed to get results from search_nearby_locations_with_nearby_location_id")?;

        if let Some(nearby_location) = nearby_location {
            let affected_location = AffectedLocationGenerator {
                affected_lines: &bare_affected_lines,
            }
            .generate(
                nearby_location.candidate,
                nearby_location.location_id.into(),
                false,
            )?;
            return Ok(Some(affected_location));
        }
        Ok(None)
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

    use super::SearcheableCandidate;

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
                    SearcheableCandidate::from(candidate.as_ref()),
                    candidate.as_str(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_location_candidate_to_candidate_copy =
            mapping_of_searcheable_location_candidate_to_candidate.clone();

        let searcheable_candidates = mapping_of_searcheable_location_candidate_to_candidate
            .keys()
            .map(|candidate| candidate.as_ref())
            .collect_vec();

        let searcheable_area_names =
            SearcheableCandidate::from_area_name(&AreaName::new(area.name.clone()));

        let directly_affected_locations = directly_affected_locations(
            db,
            &searcheable_area_names,
            &searcheable_candidates,
            time_frame.clone(),
            &mapping_of_searcheable_location_candidate_to_candidate_copy,
            source_url.clone(),
        )
        .await?;

        let potentially_affected_locations = potentially_affected_locations(
            db,
            &searcheable_area_names,
            &searcheable_candidates,
            time_frame.clone(),
            &mapping_of_searcheable_location_candidate_to_candidate_copy,
            source_url,
        )
        .await?;

        Ok(filter_out_duplicate_affected_locations(
            directly_affected_locations,
            potentially_affected_locations,
        ))
    }

    async fn directly_affected_locations(
        db: &DbAccess,
        searcheable_area_names: &[SearcheableCandidate],
        searcheable_candidates: &[&str],
        time_frame: TimeFrame,
        mapping_of_searcheable_candidate_to_original_candidate: &HashMap<
            SearcheableCandidate,
            &str,
        >,
        source: Url,
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        let pool = db.pool().await;

        let mut futures: FuturesUnordered<_> = searcheable_area_names
            .iter()
            .map(|area_name| {
                sqlx::query_as::<_, DbLocationSearchResults>(
                    "
                        SELECT * FROM location.search_locations_primary_text($1::text[], $2::text)
                        ",
                )
                .bind(searcheable_candidates)
                .bind(area_name.to_string())
                .fetch_all(pool.as_ref())
            })
            .collect();
        let mut primary_locations = vec![];
        while let Some(result) = futures.next().await {
            primary_locations.push(result.context("Failed to get primary search results from db")?);
        }
        let primary_locations = primary_locations.into_iter().flatten().collect_vec();
        let mapping_of_searcheable_candidate_to_candidate =
            mapping_of_searcheable_candidate_to_original_candidate
                .iter()
                .map(|(searcheable_candidate, original_candidate)| {
                    (
                        searcheable_candidate.to_string(),
                        original_candidate.to_string(),
                    )
                })
                .collect::<HashMap<_, _>>();
        let results = primary_locations
            .into_iter()
            .filter_map(|location| {
                mapping_of_searcheable_candidate_to_candidate
                    .get(&location.search_query)
                    .map(|original_candidate| AffectedLocation {
                        location_id: location.id.into(),
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

    async fn potentially_affected_locations(
        db: &DbAccess,
        searcheable_area_names: &[SearcheableCandidate],
        searcheable_candidates: &[&str],
        time_frame: TimeFrame,
        mapping_of_searcheable_candidate_to_original_candidate: &HashMap<
            SearcheableCandidate,
            &str,
        >,
        source: Url,
    ) -> anyhow::Result<Vec<AffectedLocation>> {
        let searcheable_candidates = searcheable_area_names
            .iter()
            .map(|name| name.to_string())
            .chain(searcheable_candidates.iter().map(ToString::to_string))
            .collect_vec();

        let mapping_of_searcheable_location_candidate_to_candidate_copy =
            mapping_of_searcheable_candidate_to_original_candidate
                .iter()
                .map(|(candidate, original_value)| (candidate, original_value.to_owned()))
                .chain(
                    searcheable_area_names
                        .iter()
                        .map(|area| (area, area.as_ref())),
                )
                .collect::<HashMap<_, _>>();

        #[derive(sqlx::FromRow, Debug)]
        pub struct NearbySearchResult {
            candidate: String,
            location_id: Uuid,
        }
        let pool = db.pool().await;
        let nearby_locations = sqlx::query_as::<_, NearbySearchResult>(
            "
                SELECT * FROM location.search_nearby_locations($1::text[])
                ",
        )
        .bind(&searcheable_candidates)
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to get nearby location search results from db")?;

        let location_ids_to_search_query = nearby_locations
            .iter()
            .map(|data| (data.location_id, data.candidate.clone()))
            .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_candidate_to_candidate =
            mapping_of_searcheable_location_candidate_to_candidate_copy
                .into_iter()
                .map(|(searcheable_candidate, original_candidate)| {
                    (searcheable_candidate.to_string(), original_candidate)
                })
                .collect::<HashMap<_, _>>();

        let results = nearby_locations
            .iter()
            .filter_map(|location| {
                location_ids_to_search_query
                    .get(&location.location_id)
                    .and_then(|candidate| {
                        mapping_of_searcheable_candidate_to_candidate
                            .get(candidate)
                            .cloned()
                            .map(|line| (line, location.location_id))
                    })
                    .map(|(line, location)| AffectedLocation {
                        location_id: location.into(),
                        line_matched: LineWithScheduledInterruptionTime {
                            line_name: line.to_string(),
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
