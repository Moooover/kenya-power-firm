use crate::subscribe_to_location::db::{BareAffectedLine, NotificationGenerator, DB};
use celery::error::TaskError;
use celery::prelude::{TaskResult, TaskResultExt};
use entities::locations::LocationId;
use entities::notifications::Notification;
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use itertools::Itertools;
use shared_kernel::uuid_key;
use sqlx::types::Json;
use std::iter;
use url::Url;
use uuid::Uuid;

use sqlx_postgres::affected_subscribers::SearcheableCandidate;

uuid_key!(NearbyLocationId);

impl DB {
    pub(super) async fn is_nearby_locations_already_fetched(
        &self,
        source_url: Url,
    ) -> TaskResult<Option<NearbyLocationId>> {
        let pool = self.pool();
        let db_results = sqlx::query!(
            r#"
            SELECT id
            FROM location.nearby_locations WHERE source_url = $1
            "#,
            source_url.to_string()
        )
        .fetch_optional(pool)
        .await
        .with_unexpected_err(|| "Failed to fetch nearby_locations")?;

        Ok(db_results.map(|record| NearbyLocationId::from(record.id)))
    }

    pub(super) async fn is_potentially_affected(
        &self,
        subscriber: SubscriberId,
        nearby_location_id: NearbyLocationId,
    ) -> TaskResult<Option<Notification>> {
        let results = BareAffectedLine::lines_affected_in_the_future(self)
            .await
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

        let bare_affected_lines = results
            .into_iter()
            .filter_map(|(area, affected_lines)| {
                affected_lines.first().cloned().map(|line| {
                    iter::once(BareAffectedLine {
                        line: area.to_string(),
                        url: line.url.clone(),
                        time_frame: line.time_frame.clone(),
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

        let nearby_location = sqlx::query_as::<_, NearbySearchResult>(
            "
            SELECT * FROM location.search_nearby_locations_with_nearby_location_id($1::text[], $2::uuid)
            ",
        )
        .bind(searcheable_candidates.clone())
        .bind(nearby_location_id.inner())
        .fetch_optional(self.pool())
        .await.with_unexpected_err(|| "Failed to get results from search_nearby_locations_with_nearby_location_id")?;

        if let Some(nearby_location) = nearby_location {
            let notification = NotificationGenerator {
                subscriber: AffectedSubscriber::PotentiallyAffected(subscriber),
                affected_lines: &bare_affected_lines,
            }
            .generate(
                nearby_location.candidate,
                nearby_location.location_id.into(),
            )
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
            return Ok(Some(notification));
        }
        Ok(None)
    }

    pub(super) async fn save_nearby_locations(
        &self,
        url: Url,
        primary_location: LocationId,
        api_response: serde_json::Value,
    ) -> TaskResult<NearbyLocationId> {
        let pool = self.pool();
        sqlx::query!(
            "
            INSERT INTO location.nearby_locations (source_url, location_id, response) 
            VALUES ($1, $2, $3) ON CONFLICT DO NOTHING
            ",
            url.to_string(),
            primary_location.inner(),
            Json(api_response) as _
        )
        .execute(pool)
        .await
        .with_unexpected_err(|| "Failed to insert nearby_locations")?;

        let record = sqlx::query!(
            r#"
            SELECT id FROM location.nearby_locations WHERE source_url = $1
            "#,
            url.to_string()
        )
        .fetch_one(pool)
        .await
        .with_unexpected_err(|| "Failed to fetch the nearby_location by id")?;

        Ok(record.id.into())
    }
}

#[cfg(test)]
pub mod tests {

    use crate::subscribe_to_location::db::DB;
    use crate::subscribe_to_location::primary_location::db_access::LocationInput;

    use entities::locations::ExternalLocationId;
    use entities::subscriptions::AffectedSubscriber;

    use serde_json::Value;

    use url::Url;

    #[tokio::test]
    async fn test_that_subscriber_is_marked_as_potentially_affected() {
        let db = DB::new_test_db().await;
        let subscriber_id = db.find_subscriber_id_created_in_fixtures().await;

        let contents = include_str!("mock_data/mi_vida_homes.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();
        let location_id = db
            .insert_location(LocationInput {
                name: "Mi Vida Homes".to_string(),
                external_id: ExternalLocationId::from("ChIJhVbiHlwVLxgRUzt5QN81vPA".to_string()),
                address: "Thika Rd, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();
        db.subscribe_to_primary_location(subscriber_id, location_id)
            .await
            .unwrap();
        let _nearby_contents = include_str!("mock_data/mi_vida_nearby_locations.json");
        let nearby_contents_value: Value = serde_json::from_str(contents).unwrap();
        let url = Url::parse("https://maps.googleapis.com/maps/api/place/nearbysearch/json?rankby=distance&location=-1.234527, 36.8769241").unwrap();
        let nearby_location_id = db
            .save_nearby_locations(url, location_id, nearby_contents_value)
            .await
            .unwrap();
        let notification = db
            .is_potentially_affected(subscriber_id, nearby_location_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            notification.subscriber,
            AffectedSubscriber::PotentiallyAffected(subscriber_id)
        );
        let line = &notification.lines.first().unwrap().line;
        assert_eq!(line, "Garden City");
    }
}
