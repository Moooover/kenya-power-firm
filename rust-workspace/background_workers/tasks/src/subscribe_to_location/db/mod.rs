use anyhow::{anyhow, Context};
use entities::locations::LocationId;
use entities::subscriptions::AffectedSubscriber;
use sqlx::PgPool;
use sqlx_postgres::repository::Repository;
use std::collections::HashMap;
use uuid::Uuid;

use chrono::Utc;
use entities::notifications::Notification;
use entities::power_interruptions::location::{
    AffectedLine, AreaName, NairobiTZDateTime, TimeFrame,
};
use itertools::Itertools;
use sqlx::types::chrono::DateTime;

use sqlx_postgres::affected_subscribers::SearcheableCandidate;
use url::Url;

use crate::configuration::REPO;

pub struct DB(Repository);

impl DB {
    pub async fn new() -> DB {
        let repo = REPO.get().await.clone();
        DB(repo)
    }

    pub fn pool(&self) -> &PgPool {
        self.0.pool()
    }
}

#[cfg(test)]
impl DB {
    pub async fn new_test_db() -> DB {
        let repo = Repository::new_test_repo().await;
        DB(repo)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BareAffectedLine {
    pub line: String,
    pub url: Url,
    pub time_frame: TimeFrame<NairobiTZDateTime>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct DbLocationSearchResults {
    pub search_query: String,
    pub location: String,
    pub id: Uuid,
}

impl BareAffectedLine {
    pub(crate) async fn lines_affected_in_the_future(
        db: &DB,
    ) -> anyhow::Result<HashMap<AreaName, Vec<Self>>> {
        #[derive(sqlx::FromRow, Debug)]
        struct DbAreaLine {
            line_name: String,
            area_name: String,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
            url: String,
        }
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
        .fetch_all(db.pool())
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

pub(crate) struct NotificationGenerator<'a> {
    pub(crate) subscriber: AffectedSubscriber,
    pub(crate) affected_lines: &'a [BareAffectedLine],
}

impl<'a> NotificationGenerator<'a> {
    pub(crate) fn generate(
        &self,
        search_query: String,
        location_id: LocationId,
    ) -> anyhow::Result<Notification> {
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
        let affected_line = AffectedLine {
            location_matched: location_id,
            line: original_line_candidate.to_string(),
            time_frame: time_frame.clone(),
        };

        let notification = Notification {
            url: url.to_owned(),
            subscriber: self.subscriber,
            lines: vec![affected_line],
        };

        Ok(notification)
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
