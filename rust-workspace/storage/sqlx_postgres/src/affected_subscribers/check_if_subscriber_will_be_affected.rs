use std::{collections::HashMap, iter::once_with};

use crate::{
    affected_subscribers::{DbLocationSearchResults, SearcheableCandidate},
    repository::Repository,
};
use anyhow::anyhow;
use anyhow::Context;
use chrono::Utc;
use chrono_tz::Tz;
use entities::notifications::Notification;
use entities::power_interruptions::location::{AffectedLine, NairobiTZDateTime, TimeFrame};
use entities::subscriptions::{AffectedSubscriber, SubscriberId};
use entities::{locations::LocationId, power_interruptions::location::AreaName};
use itertools::Itertools;
use sqlx::types::chrono::DateTime;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BareAffectedLine {
    pub line: String,
    pub url: Url,
    pub time_frame: TimeFrame<NairobiTZDateTime>,
}

impl BareAffectedLine {
    async fn lines_affected_in_the_future(
        repo: &Repository,
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
        .fetch_all(repo.pool())
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

impl Repository {
    pub async fn subscriber_directly_affected(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<Option<Notification>> {
        let results = BareAffectedLine::lines_affected_in_the_future(self).await?;
        let affected_lines = results.values().flatten().collect_vec();

        self.directly_affected_subscriber_notification(subscriber_id, location_id, &affected_lines)
            .await
    }

    async fn directly_affected_subscriber_notification(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
        affected_lines: &[&BareAffectedLine],
    ) -> anyhow::Result<Option<Notification>> {
        let pool = self.pool();

        let Mapping {
            mapping_of_line_to_time_frame,
            mapping_of_searcheble_candidate_to_original_line_candidate,
            searcheable_candidates,
            mapping_of_line_to_url,
        } = Mapping::generate(affected_lines);
        let primary_location = sqlx::query_as::<_, DbLocationSearchResults>(
            "
            SELECT * FROM location.search_specific_location_primary_text($1::text[], $2::uuid)
            ",
        )
        .bind(searcheable_candidates)
        .bind(location_id.inner())
        .fetch_optional(pool)
        .await
        .context("Failed to fetch results from search_specific_location_primary_text")?;

        if let Some(location) = primary_location {
            let original_line_candidate =
                mapping_of_searcheble_candidate_to_original_line_candidate
                    .get(&location.search_query)
                    .ok_or(anyhow!(
                        "Failed to get orinal_line_candidate from search_query {}",
                        location.search_query
                    ))?;

            let time_frame = *mapping_of_line_to_time_frame.get(original_line_candidate).ok_or(anyhow!("Failed to get time_frame when we should have for candidate {original_line_candidate}"))?;

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
                subscriber: AffectedSubscriber::DirectlyAffected(subscriber_id),
                lines: vec![affected_line],
            };

            Ok(Some(notification))
        } else {
            Ok(None)
        }
    }

    pub async fn subscriber_potentially_affected(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<Option<Notification>> {
        let mapping_of_areas_with_affected_lines =
            BareAffectedLine::lines_affected_in_the_future(self).await?;

        let map_areas_as_locations = mapping_of_areas_with_affected_lines
            .into_iter()
            .filter_map(|(area, affected_lines)| {
                let time_frame_and_url = affected_lines
                    .first()
                    .map(|line| (line.time_frame.clone(), line.url.clone()));
                time_frame_and_url.map(|(time_frame, url)| {
                    affected_lines
                        .into_iter()
                        .chain(once_with(|| BareAffectedLine {
                            line: area.inner(),
                            time_frame,
                            url,
                        }))
                        .collect_vec()
                })
            })
            .flatten()
            .collect_vec();

        let affected_lines = map_areas_as_locations.iter().collect_vec();

        let Mapping {
            mapping_of_line_to_time_frame,
            mapping_of_searcheble_candidate_to_original_line_candidate,
            searcheable_candidates,
            mapping_of_line_to_url,
        } = Mapping::generate(&affected_lines);

        let nearby_location = sqlx::query_as::<_, DbLocationSearchResults>(
            "
            SELECT * FROM location.search_specific_location_secondary_text($1::text[], $2::uuid)
            ",
        )
        .bind(searcheable_candidates)
        .bind(location_id.to_string())
        .fetch_optional(self.pool())
        .await
        .context("Failed to fetch results from search_specific_location_secondary_text")?;

        if let Some(location) = nearby_location {
            let original_line_candidate =
                mapping_of_searcheble_candidate_to_original_line_candidate
                    .get(&location.search_query)
                    .ok_or(anyhow!(
                        "Failed to get orinal_line_candidate from search_query {}",
                        location.search_query
                    ))?;

            let time_frame = *mapping_of_line_to_time_frame.get(original_line_candidate).ok_or(anyhow!("Failed to get time_frame when we should have for candidate {original_line_candidate}"))?;

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
                subscriber: AffectedSubscriber::PotentiallyAffected(subscriber_id),
                lines: vec![affected_line],
            };
            Ok(Some(notification))
        } else {
            Ok(None)
        }
    }
}

struct Mapping<'a> {
    mapping_of_line_to_time_frame: HashMap<&'a String, &'a TimeFrame<NairobiTZDateTime>>,
    mapping_of_searcheble_candidate_to_original_line_candidate: HashMap<String, &'a String>,
    searcheable_candidates: Vec<String>,
    mapping_of_line_to_url: HashMap<&'a String, &'a Url>,
}

impl<'a> Mapping<'a> {
    fn generate(affected_lines: &'a [&BareAffectedLine]) -> Self {
        let mapping_of_line_to_time_frame = affected_lines
            .iter()
            .map(|line| (&line.line, &line.time_frame))
            .collect::<HashMap<_, _>>();

        let mapping_of_line_to_url = affected_lines
            .iter()
            .map(|line| (&line.line, &line.url))
            .collect();

        let mapping_of_searcheble_candidate_to_original_line_candidate = affected_lines
            .iter()
            .map(|data| {
                (
                    SearcheableCandidate::from(data.line.as_ref()).to_string(),
                    &data.line,
                )
            })
            .collect::<HashMap<_, _>>();
        let searcheable_candidates = affected_lines
            .iter()
            .map(|data| SearcheableCandidate::from(data.line.as_ref()).to_string())
            .collect_vec();
        Self {
            mapping_of_line_to_time_frame,
            mapping_of_searcheble_candidate_to_original_line_candidate,
            searcheable_candidates,
            mapping_of_line_to_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{Days, Utc};
    use entities::{
        locations::{ExternalLocationId, LocationId},
        power_interruptions::location::{
            Area, County, ImportInput, NairobiTZDateTime, Region, TimeFrame,
        },
        subscriptions::AffectedSubscriber,
    };
    use serde_json::Value;
    use url::Url;
    use use_cases::import_affected_areas::SaveBlackoutAffectedAreasRepo;

    use crate::locations::insert_location::LocationInput;
    use crate::{affected_subscribers::tests::authenticate, repository::Repository};

    fn generate_region() -> Region {
        let tomorrow = NairobiTZDateTime::try_from(
            Utc::now()
                .naive_utc()
                .checked_add_days(Days::new(1))
                .unwrap(),
        )
        .unwrap();

        Region {
            region: "Nairobi".to_string(),
            counties: vec![County {
                name: "Nairobi".to_string(),
                areas: vec![Area {
                    name: "Garden City".to_string(),
                    time_frame: TimeFrame {
                        from: tomorrow.clone().try_into().unwrap(),
                        to: tomorrow.try_into().unwrap(),
                    },
                    locations: vec![
                        "Will Mary Estate".to_string(),
                        "Garden City Mall".to_string(),
                    ],
                }],
            }],
        }
    }

    fn generate_import_input() -> ImportInput {
        let url = Url::parse("https://example.net").unwrap();
        ImportInput(HashMap::from([(url, vec![generate_region()])]))
    }

    #[tokio::test]
    async fn test_that_subscriber_is_marked_as_directly_affected() {
        let repository = Repository::new_test_repo().await;
        let subscriber_id = authenticate(&repository).await;
        repository.save(&generate_import_input()).await.unwrap();
        let contents = include_str!("mock_data/garden_city_details_response.json");
        let api_response: Value = serde_json::from_str(contents).unwrap();
        let location_id = repository
            .insert_location(LocationInput {
                name: "Garden City Mall".to_string(),
                external_id: ExternalLocationId::from("ChIJGdueTt0VLxgRk19ir6oE8I0".to_string()),
                address: "Thika Rd, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();

        // TODO: Fix this type

        let notification = repository
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
    async fn test_that_subscriber_is_marked_as_potentially_affected() {
        let repository = Repository::new_test_repo().await;
        let subscriber_id = authenticate(&repository).await;
        let contents = include_str!("mock_data/mi_vida_homes.json");
        repository.save(&generate_import_input()).await.unwrap();

        let api_response: Value = serde_json::from_str(contents).unwrap();

        let location_id = repository
            .insert_location(LocationInput {
                name: "Mi Vida Homes".to_string(),
                external_id: ExternalLocationId::from("ChIJhVbiHlwVLxgRUzt5QN81vPA".to_string()),
                address: "Off exit, 7 Thika Rd, Nairobi, Kenya".to_string(),
                api_response,
            })
            .await
            .unwrap();

        let notification = repository
            .subscriber_potentially_affected(subscriber_id, location_id)
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