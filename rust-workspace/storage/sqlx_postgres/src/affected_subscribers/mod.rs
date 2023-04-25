use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::power_interruptions::location::{Area, NairobiTZDateTime, TimeFrame};
use entities::subscriptions::AffectedSubscriber;
use entities::{locations::LocationId, power_interruptions::location::AreaName};
use entities::{
    power_interruptions::location::{AffectedLine, FutureOrCurrentNairobiTZDateTime, Region},
    subscriptions::SubscriberId,
};
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use use_cases::notifications::notify_subscribers::SubscriberRepo;
use uuid::Uuid;

#[async_trait]
impl SubscriberRepo for Repository {
    async fn get_affected_subscribers(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine<NairobiTZDateTime>>>> {
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
            .map(|area| self.get_subscribers_with_their_affected_lines_in_an_area(area))
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

        Ok(result
            .into_iter()
            .flat_map(|v| v.into_iter().map(|(subscriber, lines)| (subscriber, lines)))
            .collect())
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

    pub fn original_value(&self) -> String {
        self.0.replace(" <-> ", " ")
    }
}

impl From<&str> for SearcheableCandidate {
    fn from(value: &str) -> Self {
        let value = value.trim().replace(' ', " <-> ");
        SearcheableCandidate(value)
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct DbLocationSearchResults {
    pub search_query: String,
    pub location: String,
    pub id: Uuid,
}

struct SubscriberWithLocation {
    subscriber: Uuid,
    location: Uuid,
}

impl Repository {
    async fn get_subscribers_with_their_affected_lines_in_an_area(
        &self,
        area: &Area<FutureOrCurrentNairobiTZDateTime>,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine<NairobiTZDateTime>>>> {
        let time_frame = area.time_frame.clone();
        let candidates = &area.locations;

        let mapping_of_searcheable_location_candidate_to_candidate = candidates
            .iter()
            .map(|candidate| {
                (
                    SearcheableCandidate::from(candidate.as_ref()),
                    candidate.as_ref(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_location_candidate_to_candidate_copy =
            mapping_of_searcheable_location_candidate_to_candidate.clone();

        let searcheable_candidates = mapping_of_searcheable_location_candidate_to_candidate
            .keys()
            .map(|candidate| candidate.as_ref())
            .collect_vec();

        let searcheable_area_names = SearcheableCandidate::from_area_name(&area.name);

        let (directly_affected_subscribers, mapping_of_subscriber_to_directly_affected_locations) =
            self.directly_affected_subscribers(
                &searcheable_area_names,
                &searcheable_candidates,
                &time_frame,
                &mapping_of_searcheable_location_candidate_to_candidate,
            )
            .await?;

        let searcheable_candidates = searcheable_area_names
            .iter()
            .map(|name| name.as_ref())
            .chain(searcheable_candidates.into_iter())
            .collect_vec();

        let mapping_of_searcheable_location_candidate_to_candidate_copy =
            mapping_of_searcheable_location_candidate_to_candidate_copy
                .into_iter()
                .map(|(candidate, original_value)| (candidate, original_value.to_owned()))
                .chain(
                    searcheable_area_names
                        .iter()
                        .map(|area| (area.clone(), area.original_value())),
                )
                .collect();

        let potentially_affected_subscribers = self
            .potentially_affected_subscribers(
                &searcheable_candidates,
                &time_frame,
                mapping_of_searcheable_location_candidate_to_candidate_copy,
                mapping_of_subscriber_to_directly_affected_locations,
            )
            .await?;

        Ok(directly_affected_subscribers
            .into_iter()
            .chain(potentially_affected_subscribers.into_iter())
            .collect())
    }

    async fn directly_affected_subscribers(
        &self,
        searcheable_area_names: &[SearcheableCandidate],
        searcheable_candidates: &[&str],
        time_frame: &TimeFrame<FutureOrCurrentNairobiTZDateTime>,
        mapping_of_searcheable_candidate_to_original_candidate: &HashMap<
            SearcheableCandidate,
            &str,
        >,
    ) -> anyhow::Result<(
        HashMap<AffectedSubscriber, Vec<AffectedLine<NairobiTZDateTime>>>,
        HashMap<Uuid, Vec<Uuid>>,
    )> {
        let pool = self.pool();
        let time_frame = TimeFrame {
            from: NairobiTZDateTime::from(&time_frame.from),
            to: NairobiTZDateTime::from(&time_frame.to),
        };

        let mut futures: FuturesUnordered<_> = searcheable_area_names
            .into_iter()
            .map(|area_name| {
                sqlx::query_as::<_, DbLocationSearchResults>(
                    "
                        SELECT * FROM location.search_locations_primary_text($1::text[], $2::text)
                        ",
                )
                .bind(searcheable_candidates)
                .bind(area_name.to_string())
                .fetch_all(pool)
            })
            .collect();
        let mut primary_locations = vec![];
        while let Some(result) = futures.next().await {
            primary_locations.push(result.context("Failed to get primary search results from db")?);
        }
        let primary_locations = primary_locations.into_iter().flatten().collect_vec();

        let location_ids = primary_locations
            .iter()
            .map(|location| location.id)
            .collect_vec();

        let directly_affected_subscribers = self
            .get_direct_subscribers_with_locations(&location_ids)
            .await
            .map(|results| {
                results
                    .into_iter()
                    .map(|data| (data.subscriber, data.location))
                    .into_group_map()
            })?;

        let location_ids_to_search_query = primary_locations
            .iter()
            .map(|data| (data.id, data.search_query.clone()))
            .collect::<HashMap<_, _>>();

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
        let result = directly_affected_subscribers
            .iter()
            .map(|(subscriber, location_ids)| {
                let locations = location_ids
                    .iter()
                    .filter_map(|location| {
                        location_ids_to_search_query
                            .get(location)
                            .and_then(|candidate| {
                                mapping_of_searcheable_candidate_to_candidate
                                    .get(candidate)
                                    .cloned()
                                    .map(|line| (line, location))
                            })
                    })
                    .map(|(line, location)| AffectedLine {
                        line,
                        location_matched: LocationId::from(*location),
                        time_frame: time_frame.clone(),
                    })
                    .collect_vec();
                (
                    AffectedSubscriber::DirectlyAffected(SubscriberId::from(*subscriber)),
                    locations,
                )
            })
            .collect::<HashMap<_, _>>();

        Ok((result, directly_affected_subscribers))
    }

    async fn potentially_affected_subscribers(
        &self,
        searcheable_candidates: &[&str],
        time_frame: &TimeFrame<FutureOrCurrentNairobiTZDateTime>,
        mapping_of_searcheable_candidate_to_original_candidate: HashMap<
            SearcheableCandidate,
            String,
        >,
        mapping_of_subscriber_to_directly_affected_locations: HashMap<Uuid, Vec<Uuid>>,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine<NairobiTZDateTime>>>> {
        let pool = self.pool();
        let time_frame = TimeFrame {
            from: NairobiTZDateTime::from(&time_frame.from),
            to: NairobiTZDateTime::from(&time_frame.to),
        };

        #[derive(sqlx::FromRow, Debug)]
        pub struct NearbySearchResult {
            candidate: String,
            location_id: Uuid,
        }

        let nearby_locations = sqlx::query_as::<_, NearbySearchResult>(
            "
                SELECT * FROM location.search_nearby_locations($1::text[])
                ",
        )
        .bind(&searcheable_candidates)
        .fetch_all(pool)
        .await
        .context("Failed to get nearby location search results from db")?;

        let location_ids = nearby_locations
            .iter()
            .map(|location| location.location_id)
            .collect_vec();

        let potentially_affected_subscribers = self
            .get_potentially_affected_subscribers(pool, &location_ids)
            .await?;

        let potentially_affected_subscribers =
            filter_out_directly_affected_subscriber_locations_from_potentially_affected(
                mapping_of_subscriber_to_directly_affected_locations,
                potentially_affected_subscribers,
            );

        let location_ids_to_search_query = nearby_locations
            .iter()
            .map(|data| (data.location_id, data.candidate.clone()))
            .collect::<HashMap<_, _>>();

        let mapping_of_searcheable_candidate_to_candidate =
            mapping_of_searcheable_candidate_to_original_candidate
                .into_iter()
                .map(|(searcheable_candidate, original_candidate)| {
                    (searcheable_candidate.to_string(), original_candidate)
                })
                .collect::<HashMap<_, _>>();
        let result = potentially_affected_subscribers
            .iter()
            .map(|(subscriber, location_ids)| {
                let locations = location_ids
                    .iter()
                    .filter_map(|location| {
                        location_ids_to_search_query
                            .get(location)
                            .and_then(|candidate| {
                                mapping_of_searcheable_candidate_to_candidate
                                    .get(candidate)
                                    .cloned()
                                    .map(|line| (line, location))
                            })
                    })
                    .map(|(line, location)| AffectedLine {
                        line,
                        location_matched: LocationId::from(*location),
                        time_frame: time_frame.clone(),
                    })
                    .collect_vec();
                (
                    AffectedSubscriber::PotentiallyAffected(SubscriberId::from(*subscriber)),
                    locations,
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(result)
    }

    async fn get_potentially_affected_subscribers(
        &self,
        pool: &PgPool,
        location_ids: &[Uuid],
    ) -> anyhow::Result<HashMap<Uuid, Vec<Uuid>>> {
        // TODO: Set search path on pool;
        let potentially_affected_subscribers = sqlx::query!(
            "
            SELECT subscriber_id, adjuscent_location_id FROM location.adjuscent_locations 
            INNER JOIN location.subscriber_locations ON location.adjuscent_locations.initial_location_id = location.subscriber_locations.id 
            WHERE location.adjuscent_locations.adjuscent_location_id = ANY($1)
            ",
                &location_ids[..]
            ).fetch_all(pool).await.context("Failed to get subscribers subscribed to nearby locations")?;

        // We are still getting direct subscriber locations
        // because some subscribers might be directly subscribed to the location
        // however, we are still marking them as PottentiallyAffected because
        // we scanned the text in the API-Response via `search_locations_secondary_text()`
        let directly_affected_subscribers = self
            .get_direct_subscribers_with_locations(location_ids)
            .await?;

        Ok(potentially_affected_subscribers
            .into_iter()
            .map(|record| (record.subscriber_id, record.adjuscent_location_id))
            .chain(
                directly_affected_subscribers
                    .into_iter()
                    .map(|data| (data.subscriber, data.location)),
            )
            .into_group_map())
    }

    async fn get_direct_subscribers_with_locations(
        &self,
        location_ids: &[Uuid],
    ) -> anyhow::Result<Vec<SubscriberWithLocation>> {
        let pool = self.pool();
        let primary_affected_subscribers = sqlx::query!(
            "
            SELECT subscriber_id, location_id FROM location.subscriber_locations WHERE location_id = ANY($1)
            ",
            &location_ids[..]
        ).fetch_all(pool).await.context("Failed to get subscribers subscribed to primary locations")?;

        Ok(primary_affected_subscribers
            .into_iter()
            .map(|record| SubscriberWithLocation {
                subscriber: record.subscriber_id,
                location: record.location_id,
            })
            .collect_vec())
    }
}

fn filter_out_directly_affected_subscriber_locations_from_potentially_affected(
    mapping_of_subscriber_to_directly_affected_locations: HashMap<Uuid, Vec<Uuid>>,
    mapping_of_subscriber_to_potentially_affected_locations: HashMap<Uuid, Vec<Uuid>>,
) -> HashMap<Uuid, Vec<Uuid>> {
    mapping_of_subscriber_to_potentially_affected_locations
        .into_iter()
        .filter_map(|(subscriber_id, nearby_location_ids)| {
            let primary_subscribers = mapping_of_subscriber_to_directly_affected_locations
                .get(&subscriber_id)
                .cloned()
                .unwrap_or_default();
            let nearby_location_ids: HashSet<_> =
                HashSet::from_iter(nearby_location_ids.into_iter());

            let locations_not_in_primary_locations: Vec<_> = nearby_location_ids
                .difference(&HashSet::from_iter(primary_subscribers.into_iter()))
                .cloned()
                .collect();
            if locations_not_in_primary_locations.is_empty() {
                None
            } else {
                Some((subscriber_id, locations_not_in_primary_locations))
            }
        })
        .collect::<HashMap<_, _>>()
}

mod test_fixtures {
    use crate::repository::Repository;
    use entities::locations::{ExternalLocationId, LocationId};
    use entities::subscriptions::SubscriberId;
    use sqlx::types::Json;
    use url::Url;

    pub struct LocationInput {
        name: String,
        address: String,
        external_id: ExternalLocationId,
        api_response: serde_json::Value,
    }
    impl Repository {
        pub async fn insert_location_and_subscribe(
            &self,
            location: LocationInput,
            subscriber: SubscriberId,
        ) -> LocationId {
            let external_id = location.external_id.as_ref();
            let address = location.address.clone();
            struct Location {
                id: uuid::Uuid,
            }
            let result = sqlx::query!(
            "
            INSERT INTO location.locations (name, external_id, address, sanitized_address, external_api_response)
            VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING RETURNING id
            ",
            location.name,
            external_id,
            address.clone(),
            address,
            Json(location.api_response) as _
        )
        .fetch_one(self.pool())
        .await.unwrap();

            let location_id: LocationId = result.id.into();

            let _ = sqlx::query!(
                r#"
              INSERT INTO location.subscriber_locations (subscriber_id, location_id)
              VALUES ($1, $2) ON CONFLICT DO NOTHING
            "#,
                subscriber.inner(),
                location_id.inner()
            )
            .execute(self.pool())
            .await
            .unwrap();

            location_id
        }

        pub async fn save_nearby_locations(
            &self,
            primary_location: LocationId,
            api_response: serde_json::Value,
        ) {
            let pool = self.pool();
            let url =
                Url::parse("https://www.kplc.co.ke/img/full/Interruption%20-%2022.12.2022.pdf")
                    .unwrap();
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
            .unwrap();
        }
    }

    #[cfg(test)]
    mod test {
        use crate::affected_subscribers::test_fixtures::LocationInput;
        use crate::fixtures::{nairobi_region, SUBSCRIBER_EXTERNAL_ID};
        use crate::repository::Repository;

        use entities::subscriptions::AffectedSubscriber;
        use serde_json::Value;
        use use_cases::notifications::notify_subscribers::SubscriberRepo;

        #[tokio::test]
        async fn test_directly_affected_subscriber_works() {
            let repo = Repository::new_test_repo().await;

            let subscriber = repo
                .find_by_external_id(SUBSCRIBER_EXTERNAL_ID.clone())
                .await
                .unwrap();
            let contents = include_str!("mock_data/garden_city_mall.json");
            let api_response: Value = serde_json::from_str(contents).unwrap();
            let location = LocationInput {
                name: "Garden city mall".to_string(),
                address: "Thika Road, Nairobi, Kenya".to_string(),
                external_id: "ChIJGdueTt0VLxgRk19ir6oE8I0".into(),
                api_response,
            };
            let _ = repo
                .insert_location_and_subscribe(location, subscriber)
                .await;

            let result = repo
                .get_affected_subscribers(&[nairobi_region()])
                .await
                .unwrap();

            assert!(result
                .get(&AffectedSubscriber::DirectlyAffected(subscriber))
                .is_some())
        }

        #[tokio::test]
        async fn test_potentially_affected_subscriber_works() {
            let repo = Repository::new_test_repo().await;

            let subscriber = repo
                .find_by_external_id(SUBSCRIBER_EXTERNAL_ID.clone())
                .await
                .unwrap();
            let contents = include_str!("mock_data/mi_vida_homes.json");
            let api_response: Value = serde_json::from_str(contents).unwrap();
            let location = LocationInput {
                name: "Mi vida Homes".to_string(),
                address: "Thika Road, Nairobi, Kenya".to_string(),
                external_id: "ChIJhVbiHlwVLxgRUzt5QN81vPA".into(),
                api_response,
            };
            let location = repo
                .insert_location_and_subscribe(location, subscriber)
                .await;

            let contents = include_str!("mock_data/nearby_mi_vida_homes.json");
            let api_response: Value = serde_json::from_str(contents).unwrap();
            repo.save_nearby_locations(location, api_response).await;

            let result = repo
                .get_affected_subscribers(&[nairobi_region()])
                .await
                .unwrap();

            assert!(result
                .get(&AffectedSubscriber::PotentiallyAffected(subscriber))
                .is_some())
        }
    }
}
