use crate::subscribe_to_location::db::{
    BareAffectedLine, DbLocationSearchResults, NotificationGenerator, DB,
};

use anyhow::Context;
use celery::prelude::TaskResultExt;
use celery::{prelude::TaskError, task::TaskResult};

use entities::locations::ExternalLocationId;
use entities::notifications::Notification;

use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use entities::{locations::LocationId, power_interruptions::location::AreaName};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex, RegexBuilder};
use serde::Deserialize;
use sqlx_postgres::affected_subscribers::SearcheableCandidate;

use sqlx::types::Json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use uuid::Uuid;

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

pub struct LocationWithCoordinates {
    pub location_id: LocationId,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone)]
pub struct LocationInput {
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}

impl DB {
    pub async fn insert_location(&self, location: LocationInput) -> TaskResult<LocationId> {
        let pool = self.pool();
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
        .execute(pool)
        .await
        .with_unexpected_err(|| "Failed to insert location")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.locations WHERE external_id = $1
            "#,
            external_id
        )
        .fetch_one(pool)
        .await
        .with_unexpected_err(|| "Failed to get inserted location")?;

        Ok(record.id.into())
    }
    pub async fn subscribe_to_primary_location(
        &self,
        subscriber: SubscriberId,
        location_id: LocationId,
    ) -> TaskResult<Uuid> {
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
        .with_unexpected_err(|| "Failed to insert subscriber_location")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.subscriber_locations WHERE subscriber_id = $1 AND location_id = $2
            "#,
             subscriber,
              location_id
        ).fetch_one(self.pool()).await.with_unexpected_err(|| "Failed to return id of subscribed location")?;

        Ok(record.id)
    }

    pub async fn find_location_id_and_coordinates(
        &self,
        location: ExternalLocationId,
    ) -> TaskResult<Option<LocationWithCoordinates>> {
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
        let result = sqlx::query_as!(
            Row,
            r#"
            SELECT id, external_api_response as "value: Json<ResultWrapper>" FROM location.locations WHERE external_id = $1
            "#,
            location.inner()
        )
        .fetch_optional(self.pool())
        .await.with_unexpected_err(|| {
            format!("Failed to get response from FROM location.locations WHERE external_id = {}", location.inner())
        })?;

        let result = result.map(|data| LocationWithCoordinates {
            location_id: data.id.into(),
            latitude: data.value.result.geometry.location.lat,
            longitude: data.value.result.geometry.location.lng,
        });

        Ok(result)
    }

    pub async fn subscriber_directly_affected(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> TaskResult<Option<Notification>> {
        let results = BareAffectedLine::lines_affected_in_the_future(self)
            .await
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
        for (area_name, affected_lines) in results.iter() {
            let notification = self
                .directly_affected_subscriber_notification(
                    subscriber_id,
                    location_id,
                    area_name,
                    affected_lines,
                )
                .await?;
            if let Some(notification) = notification {
                return Ok(Some(notification));
            }
        }

        Ok(None)
    }

    async fn directly_affected_subscriber_notification(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
        area_name: &AreaName,
        affected_lines: &[BareAffectedLine],
    ) -> TaskResult<Option<Notification>> {
        let pool = self.pool();

        let searcheable_candidates = affected_lines
            .iter()
            .map(|line| SearcheableCandidate::from(line.line.as_ref()).to_string())
            .collect_vec();

        let primary_location = Self::get_primary_location_search_result(
            location_id,
            area_name,
            pool,
            searcheable_candidates,
        )
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        if let Some(location) = primary_location {
            let notification = NotificationGenerator {
                subscriber: AffectedSubscriber::DirectlyAffected(subscriber_id),
                affected_lines,
            }
            .generate(location.search_query, location.id.into())
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

            Ok(Some(notification))
        } else {
            Ok(None)
        }
    }

    async fn get_primary_location_search_result(
        location_id: LocationId,
        area_name: &AreaName,
        pool: &PgPool,
        searcheable_candidates: Vec<String>,
    ) -> anyhow::Result<Option<DbLocationSearchResults>> {
        let mut primary_location: Option<DbLocationSearchResults> = None;

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
                .fetch_optional(pool)
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

#[cfg(test)]
pub mod tests {

    use crate::subscribe_to_location::db::DB;
    use crate::subscribe_to_location::primary_location::db_access::{
        LocationInput, NonAcronymString,
    };

    use entities::locations::ExternalLocationId;
    use entities::subscriptions::{AffectedSubscriber, SubscriberId};

    use serde_json::Value;
    use sqlx_postgres::fixtures::SUBSCRIBER_EXTERNAL_ID;

    use uuid::Uuid;

    impl DB {
        pub async fn find_subscriber_id_created_in_fixtures(&self) -> SubscriberId {
            let external_id = SUBSCRIBER_EXTERNAL_ID.as_ref();
            #[derive(sqlx::FromRow, Debug)]
            struct Subscriber {
                id: Uuid,
            }
            let result = sqlx::query_as::<_, Subscriber>(&format!(
                "SELECT id FROM public.subscriber WHERE external_id = '{}'",
                external_id
            ))
            .fetch_one(self.pool())
            .await
            .unwrap();

            result.id.into()
        }
    }

    #[tokio::test]
    async fn test_that_subscriber_is_marked_as_directly_affected() {
        let db = DB::new_test_db().await;
        let subscriber_id = db.find_subscriber_id_created_in_fixtures().await;

        let contents = include_str!("../db/mock_data/garden_city_details_response.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();
        let location_id = db
            .insert_location(LocationInput {
                name: "Garden City Mall".to_string(),
                external_id: ExternalLocationId::from("ChIJGdueTt0VLxgRk19ir6oE8I0".to_string()),
                address: "Thika Rd, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();
        db.subscribe_to_primary_location(subscriber_id, location_id)
            .await
            .unwrap();
        let notification = db
            .subscriber_directly_affected(subscriber_id, location_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            notification.subscriber,
            AffectedSubscriber::DirectlyAffected(subscriber_id)
        );
        let line = &notification.lines.first().unwrap().line;
        assert_eq!(line, "Garden City Mall");
    }

    #[tokio::test]
    async fn test_locations_with_generic_names_are_first_matched_to_area() {
        let db = DB::new_test_db().await;
        let subscriber_id = db.find_subscriber_id_created_in_fixtures().await;

        let contents = include_str!("../db/mock_data/generic_name/citam_nairobi_pentecostal.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();

        let location_id = db
            .insert_location(LocationInput {
                name: "Nairobi Pentecostal Church".to_string(),
                external_id: ExternalLocationId::from("ChIJhVbiHlwVLxgRUzt5QN81vPA".to_string()),
                address: "CITAM BURU BURU PREMISES,NZIU, Starehe, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();

        db.subscribe_to_primary_location(subscriber_id, location_id)
            .await
            .unwrap();

        let contents = include_str!("../db/mock_data/generic_name/fishermens_pentecostal.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();

        let location_id = db
            .insert_location(LocationInput {
                name: "Fisher's of men Pentecostal church".to_string(),
                external_id: ExternalLocationId::from("ChIJwxGb7pVrLxgRdxwwMVASs-c".to_string()),
                address: "PXV6+JVG, Kangundo Rd, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();
        db.subscribe_to_primary_location(subscriber_id, location_id)
            .await
            .unwrap();

        let contents = include_str!("../db/mock_data/generic_name/victory_pentecostal_church.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();

        let location_id = db
            .insert_location(LocationInput {
                name: "Victory Pentecostal Church (Jabez Experience Centre)".to_string(),
                external_id: ExternalLocationId::from("ChIJSx8C4LERLxgR-ml5tyOEkPw".to_string()),
                address: "MQRQ+GVF, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();
        db.subscribe_to_primary_location(subscriber_id, location_id)
            .await
            .unwrap();

        let notification = db
            .subscriber_directly_affected(subscriber_id, location_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            notification.subscriber,
            AffectedSubscriber::DirectlyAffected(subscriber_id)
        );
    }

    #[test]
    fn test_acronym_new_type() {
        let value = "Garden city Rd";
        let value = NonAcronymString::from(value.to_string());
        let expected_value = "Garden city Road";
        assert_eq!(value.to_string(), expected_value.to_string())
    }

    #[test]
    fn test_acronym_with_list() {
        let input_with_expected_result = vec![
            ("Garden city Rd", "Garden city Road"),
            ("Sombogo T/Fact", "Sombogo Tea Factory"),
            ("DCI HQtrs", "DCI Headquaters"),
        ];

        input_with_expected_result
            .iter()
            .for_each(|(input, expected_result)| {
                let value = NonAcronymString::from(input.to_string());

                assert_eq!(value.to_string(), expected_result.to_string())
            })
    }
}
