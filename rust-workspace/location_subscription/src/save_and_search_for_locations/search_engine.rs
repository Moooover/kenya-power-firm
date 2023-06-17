use entities::locations::{ExternalLocationId, LocationId};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

use self::algolia_search_engine::AlgoliaClient;

use super::NearbyLocationId;

#[derive(Debug, Deserialize, Serialize)]
pub struct LocationDTO {
    pub id: LocationId,
    #[serde(rename = "objectID")]
    pub object_id: LocationId,
    pub name: String,
    pub external_id: ExternalLocationId,
    pub address: String,
    pub api_response: serde_json::Value,
}

pub mod import_primary_locations {
    use anyhow::anyhow;
    use anyhow::Context;
    use itertools::Itertools;

    use crate::{
        db_access::DbAccess,
        save_and_search_for_locations::search_engine::{
            save_primary_location::PRIMARY_LOCATIONS_INDEX, SearchEngine,
        },
    };

    use super::LocationDTO;

    #[tracing::instrument(err, level = "info")]
    async fn fetch_all() -> anyhow::Result<Vec<LocationDTO>> {
        let db = DbAccess;
        let pool = db.pool().await;
        let results = sqlx::query!(
            "
            SELECT id, name, external_id, sanitized_address, external_api_response FROM location.locations
            "
        ).fetch_all(pool.as_ref()).await.map_err(|err| {
            anyhow!("Failed to fetch all locations {}", err)
        })?;

        let results = results
            .into_iter()
            .map(|data| LocationDTO {
                id: data.id.into(),
                object_id: data.id.into(),
                name: data.name,
                external_id: data.external_id.into(),
                address: data.sanitized_address,
                api_response: data.external_api_response,
            })
            .collect_vec();
        Ok(results)
    }

    #[tracing::instrument(err, level = "info")]
    pub async fn execute() -> anyhow::Result<()> {
        let engine = SearchEngine::new();
        let values = fetch_all()
            .await?
            .into_iter()
            .map(|item| serde_json::to_value(item).context("Failed to convert to json"))
            .collect::<Result<Vec<_>, _>>()?;

        engine.import(PRIMARY_LOCATIONS_INDEX, values).await
    }
}

pub mod save_primary_location {
    use anyhow::Context;
    use entities::locations::{ExternalLocationId, LocationId};

    use crate::save_and_search_for_locations::search_engine::{LocationDTO, SearchEngine};
    pub const PRIMARY_LOCATIONS_INDEX: &str = "primary_locations";

    #[tracing::instrument(err, level = "info")]
    pub async fn execute(
        id: LocationId,
        name: String,
        external_id: ExternalLocationId,
        address: String,
        api_response: serde_json::Value,
    ) -> anyhow::Result<()> {
        let location = LocationDTO {
            id,
            object_id: id,
            name,
            external_id,
            address,
            api_response,
        };
        let body = serde_json::to_value(location).context("Failed to convert to json")?;
        let search_engine = SearchEngine::new();
        search_engine
            .save_object(PRIMARY_LOCATIONS_INDEX, body)
            .await
    }
}

pub mod directly_affected_area_locations {
    use std::collections::HashMap;

    use entities::locations::LocationId;
    use futures::{stream::FuturesUnordered, StreamExt};
    use itertools::Itertools;
    use shared_kernel::area_name::AreaName;

    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableAreaName;

    use super::{
        save_primary_location::PRIMARY_LOCATIONS_INDEX, LocationDTO,
        SearchEngine as SearchEngineInner,
    };

    pub struct DirectlyAffectedLocationsSearchEngine {
        search_engine: SearchEngineInner,
        area_name: AreaName,
    }

    impl DirectlyAffectedLocationsSearchEngine {
        pub fn new(area_name: AreaName) -> Self {
            Self {
                search_engine: SearchEngineInner::new(),
                area_name,
            }
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn search(
            &self,
            items: Vec<String>,
        ) -> anyhow::Result<HashMap<LocationId, String>> {
            let searcheable_area_names = SearcheableAreaName::new(&self.area_name);
            let searcheable_area_names = searcheable_area_names.into_inner();
            let mapping_of_searcheable_item_to_item = searcheable_area_names
                .into_iter()
                .flat_map(|area| {
                    items
                        .iter()
                        .map(move |item| (format!("{} {}", item, &area), item.to_owned()))
                })
                .collect::<HashMap<_, _>>();

            let searcheable_items = mapping_of_searcheable_item_to_item
                .keys()
                .cloned()
                .collect_vec();
            let mut results = vec![];
            let mut futures: FuturesUnordered<_> = searcheable_items
                .into_iter()
                .map(|query| {
                    self.search_engine
                        .search::<LocationDTO, String>(PRIMARY_LOCATIONS_INDEX, query)
                })
                .collect();

            while let Some(result) = futures.next().await {
                results.push(result?);
            }
            let results = results
                .into_iter()
                .filter_map(|(query, data)| {
                    mapping_of_searcheable_item_to_item.get(&query).map(|item| {
                        data.into_iter()
                            .map(|primary_location| (primary_location.id, item.clone()))
                    })
                })
                .flatten()
                .collect::<HashMap<_, _>>();
            Ok(results)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct NearbyLocationDTO {
    pub id: NearbyLocationId,
    #[serde(rename = "objectID")]
    pub object_id: NearbyLocationId,
    pub location_id: LocationId,
    pub api_response: serde_json::Value,
}

pub mod import_nearby_locations {
    use anyhow::Context;
    use itertools::Itertools;

    use crate::{
        db_access::DbAccess, save_and_search_for_locations::search_engine::NearbyLocationDTO,
    };

    use super::{save_nearby_location::NEARBY_LOCATIONS_INDEX, SearchEngine};

    async fn fetch_all() -> anyhow::Result<Vec<NearbyLocationDTO>> {
        let db = DbAccess;
        let pool = db.pool().await;

        let results = sqlx::query!(
            "
            SELECT id, location_id, response FROM location.nearby_locations
            "
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to fetch all records from nearby_locations")?;
        let results = results
            .into_iter()
            .map(|result| NearbyLocationDTO {
                id: result.id.into(),
                object_id: result.id.into(),
                location_id: result.location_id.into(),
                api_response: result.response,
            })
            .collect_vec();

        Ok(results)
    }

    #[tracing::instrument(err, level = "info")]
    pub async fn execute() -> anyhow::Result<()> {
        let engine = SearchEngine::new();
        let values = fetch_all()
            .await?
            .into_iter()
            .map(|item| serde_json::to_value(item).context("Failed to convert to json"))
            .collect::<Result<Vec<_>, _>>()?;

        engine.import(NEARBY_LOCATIONS_INDEX, values).await
    }
}

pub mod save_nearby_location {
    use anyhow::Context;
    use entities::locations::LocationId;

    use crate::save_and_search_for_locations::NearbyLocationId;

    pub const NEARBY_LOCATIONS_INDEX: &str = "nearby_locations";

    use super::{NearbyLocationDTO, SearchEngine};

    #[tracing::instrument(err, level = "info")]
    pub async fn execute(
        primary_location: LocationId,
        api_response: serde_json::Value,
        nearby_location_id: NearbyLocationId,
    ) -> anyhow::Result<()> {
        let data = NearbyLocationDTO {
            id: nearby_location_id,
            object_id: nearby_location_id,
            location_id: primary_location,
            api_response,
        };
        let body = serde_json::to_value(data).context("Failed to convert to json")?;
        let search_engine = SearchEngine::new();
        search_engine
            .save_object(NEARBY_LOCATIONS_INDEX, body)
            .await
    }
}

pub mod potentially_affected_area_locations {
    use std::collections::HashMap;

    use entities::locations::LocationId;
    use futures::{stream::FuturesUnordered, StreamExt};
    use itertools::Itertools;
    use shared_kernel::area_name::AreaName;

    use crate::save_and_search_for_locations::searcheable_candidate::SearcheableAreaName;

    use super::{
        save_nearby_location::NEARBY_LOCATIONS_INDEX, NearbyLocationDTO,
        SearchEngine as SearchEngineInner,
    };

    pub struct NearbyLocationsSearchEngine {
        search_engine: SearchEngineInner,
        area_name: AreaName,
    }

    impl NearbyLocationsSearchEngine {
        pub fn new(area_name: AreaName) -> Self {
            Self {
                search_engine: SearchEngineInner::new(),
                area_name,
            }
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn search(
            &self,
            items: Vec<String>,
        ) -> anyhow::Result<HashMap<LocationId, String>> {
            let searcheable_area_names = SearcheableAreaName::new(&self.area_name);
            let searcheable_area_names = searcheable_area_names.into_inner();
            let mapping_of_searcheable_item_to_item = searcheable_area_names
                .into_iter()
                .flat_map(|area| {
                    items
                        .iter()
                        .map(move |item| (format!("{} {}", item, &area), item.to_owned()))
                })
                .collect::<HashMap<_, _>>();

            let searcheable_items = mapping_of_searcheable_item_to_item
                .keys()
                .cloned()
                .collect_vec();
            let mut results = vec![];
            let mut futures: FuturesUnordered<_> = searcheable_items
                .into_iter()
                .map(|query| {
                    self.search_engine
                        .search::<NearbyLocationDTO, String>(NEARBY_LOCATIONS_INDEX, query)
                })
                .collect();

            while let Some(result) = futures.next().await {
                results.push(result?);
            }
            let results = results
                .into_iter()
                .filter_map(|(query, data)| {
                    mapping_of_searcheable_item_to_item.get(&query).map(|item| {
                        data.into_iter()
                            .map(|nearby_location| (nearby_location.location_id, item.clone()))
                    })
                })
                .flatten()
                .collect::<HashMap<_, _>>();
            Ok(results)
        }
    }
}

pub struct SearchEngine {
    client: AlgoliaClient,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            client: AlgoliaClient::new(),
        }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn save_object(
        &self,
        index: impl ToString + Debug,
        body: Value,
    ) -> anyhow::Result<()> {
        self.client.post(index.to_string(), body).await
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn search<DTO: DeserializeOwned + Debug, Q: ToString + Debug>(
        &self,
        index: impl ToString + Debug,
        query: Q,
    ) -> anyhow::Result<(Q, Vec<DTO>)> {
        let response = self
            .client
            .get::<DTO>(index.to_string(), query.to_string())
            .await?;

        Ok((query, response))
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn import(
        &self,
        index: impl ToString + Debug,
        data: Vec<Value>,
    ) -> anyhow::Result<()> {
        self.client.import(index.to_string(), data).await
    }
}

mod algolia_search_engine {
    use anyhow::{anyhow, bail, Context};

    use itertools::Itertools;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::Value;
    use shared_kernel::non_empty_string;
    use shared_kernel::{http_client::HttpClient, string_key};
    use std::{collections::HashMap, fmt::Debug, iter};
    use tracing::{error, warn};
    use url::Url;

    use crate::config::SETTINGS_CONFIG;
    string_key!(APIKey);
    string_key!(ApplicationId);
    non_empty_string!(IndexName);
    non_empty_string!(Query);

    const NUMBER_OF_ITEMS_PER_REQUEST: usize = 100;

    //The primary hosts are {Application-ID}.algolia.net for write operations and {Application-ID}-dsn.algolia.net for read operations.
    // The *-dsn host guarantees high availability through automatic load balancing and also leverages the Distributed Search Network (if you subscribed that option).
    // To guarantee a high availability, you should implement a retry strategy for all API calls on the following fallback hosts:
    // {Application-ID}-1.algolianet.com, {Application-ID}-2.algolianet.com, {Application-ID}-3.algolianet.com.
    // (Note that the domain is different because it’s hosted on another DNS provider, to increase reliability).
    // It’s best to shuffle (randomize) the list of fallback hosts to ensure load balancing across clients. All Algolia API clients implement this retry strategy.
    struct Hosts {
        read_hosts: Vec<String>,
        write_hosts: Vec<String>,
    }

    impl Hosts {
        fn new(application_id: ApplicationId) -> Self {
            let fallback_hosts = (1..=3)
                .into_iter()
                .map(|item| format!("{}-{item}.algolianet.com", &application_id))
                .collect_vec();
            let primary_read_host = format!("{}-dsn.algolia.net", &application_id);
            let primary_write_host = format!("{}.algolia.net", &application_id);
            Self {
                read_hosts: iter::once(primary_read_host)
                    .chain(fallback_hosts.iter().cloned())
                    .collect_vec(),
                write_hosts: iter::once(primary_write_host)
                    .chain(fallback_hosts.into_iter())
                    .collect_vec(),
            }
        }
    }

    #[derive(Deserialize, Debug)]
    struct PostResponse {
        #[serde(rename = "taskID")]
        task_id: u64,
    }

    struct AlgoliaHeaders(HashMap<&'static str, String>);

    impl AlgoliaHeaders {
        fn new(api_key: APIKey, application_id: ApplicationId) -> Self {
            let headers = HashMap::from([
                ("X-Algolia-API-Key", api_key.to_string()),
                ("X-Algolia-Application-Id", application_id.to_string()),
            ]);
            Self(headers)
        }

        fn inner(&self) -> HashMap<&'static str, String> {
            self.0.clone()
        }
    }

    pub struct AlgoliaClient {
        hosts: Hosts,
        headers: AlgoliaHeaders,
    }

    impl AlgoliaClient {
        pub fn new() -> Self {
            let api_key = SETTINGS_CONFIG.search_engine.api_key.clone().into();
            let application_id: ApplicationId =
                SETTINGS_CONFIG.search_engine.application_key.clone().into();
            let headers = AlgoliaHeaders::new(api_key, application_id.clone());
            Self {
                hosts: Hosts::new(application_id),
                headers,
            }
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn post(
            &self,
            index: impl TryInto<IndexName, Error = String> + Debug,
            body: Value,
        ) -> anyhow::Result<()> {
            let index = index.try_into().map_err(|err| anyhow!(err))?;
            let urls = self
                .hosts
                .write_hosts
                .iter()
                .map(|host| {
                    Url::parse(&format!("https://{}/1/indexes/{}", host, index))
                        .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;
            let mut errors = vec![];

            for url in urls {
                let result =
                    HttpClient::post_json::<PostResponse>(url, self.headers.inner(), body.clone())
                        .await;
                match result {
                    Ok(progress) => return self.post_progress(index, progress).await,
                    Err(err) => {
                        warn!("failed to get response from POST request {:?}", err);
                        errors.push(err)
                    }
                }
            }
            error!(
                "Errors from inserting data from index {} are {:?}",
                &index, errors
            );
            bail!("Failed to return response from algolia")
        }

        async fn post_progress(
            &self,
            index: IndexName,
            progress: PostResponse,
        ) -> anyhow::Result<()> {
            let urls = self
                .hosts
                .read_hosts
                .iter()
                .map(|read_host| {
                    Url::parse(&format!(
                        "https://{}/1/indexes/{}/task/{}",
                        read_host, &index, progress.task_id
                    ))
                    .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            #[derive(Deserialize, Debug)]
            enum Status {
                #[serde(rename = "published")]
                Published,
                #[serde(rename = "notPublished")]
                NotPublished,
            }
            #[derive(Deserialize, Debug)]
            struct TaskResponse {
                status: Status,
            }
            let mut errors = vec![];
            for url in urls {
                let maybe_err = loop {
                    let response = HttpClient::get_json_with_headers::<TaskResponse>(
                        url.clone(),
                        self.headers.inner(),
                    )
                    .await;
                    match response {
                        Ok(data) if matches!(data.status, Status::Published) => {
                            break None;
                        }
                        Ok(_) => {
                            continue;
                        }
                        Err(err) => {
                            error!("Error when checking status of task {}", &err);
                            break Some(err);
                        }
                    }
                };
                if let Some(err) = maybe_err {
                    errors.push(err);
                } else {
                    break;
                }
            }

            if !errors.is_empty() {
                error!(
                    "All errors {:?} from checking status of task = {:?}",
                    &errors, &index
                );
                bail!(
                    "Failed to get status of task {}. Errors {:?}",
                    &index,
                    &errors
                )
            }

            Ok(())
        }

        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn get<DTO: DeserializeOwned + Debug>(
            &self,
            index: impl TryInto<IndexName, Error = String> + Debug,
            query: impl TryInto<Query, Error = String> + Debug,
        ) -> Result<Vec<DTO>, anyhow::Error> {
            let index = index.try_into().map_err(|err| anyhow!(err))?;
            let query = query.try_into().map_err(|err| anyhow!(err))?;

            let urls = self
                .hosts
                .read_hosts
                .iter()
                .map(|host| {
                    Url::parse_with_params(
                        &format!("https://{}/1/indexes/{}", host, &index),
                        &[("query", query.clone())],
                    )
                    .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            let mut errors = vec![];

            #[derive(Deserialize, Debug)]
            struct HitsResponse<DTO> {
                hits: Vec<DTO>,
            }

            for url in urls {
                let result = HttpClient::get_json_with_headers::<HitsResponse<DTO>>(
                    url,
                    self.headers.inner(),
                )
                .await;
                match result {
                    Ok(res) => return Ok(res.hits),
                    Err(err) => {
                        warn!("failed to get response {err:?}");
                        errors.push(err);
                    }
                }
            }
            error!(
                "Errors from index {:?} & query {} are {:?}",
                &index, &query, errors
            );
            bail!("Failed to return response from algolia")
        }

        // https://www.algolia.com/doc/rest-api/search/#batch-write-operations
        #[tracing::instrument(err, skip(self), level = "info")]
        pub async fn import(
            &self,
            index: impl TryInto<IndexName, Error = String> + Debug,
            data: Vec<Value>,
        ) -> anyhow::Result<()> {
            let index = index.try_into().map_err(|err| anyhow!(err))?;
            let urls = self
                .hosts
                .write_hosts
                .iter()
                .map(|host| {
                    Url::parse(&format!("https://{}/1/indexes/{}/batch", host, index))
                        .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            #[derive(Serialize, Debug)]
            enum RequestAction {
                #[serde(rename = "addObject")]
                AddObject,
            }

            #[derive(Serialize, Debug)]
            struct Request {
                action: RequestAction,
                body: Value,
            }

            #[derive(Serialize, Debug)]
            struct RequestBody {
                requests: Vec<Request>,
            }

            let mut errors = vec![];

            #[derive(Deserialize)]
            #[allow(dead_code)]
            struct ImportResponse {
                #[serde(rename = "objectIDs")]
                object_ids: Vec<String>,
            }

            for chunk in data.chunks(NUMBER_OF_ITEMS_PER_REQUEST) {
                let request = RequestBody {
                    requests: chunk
                        .iter()
                        .map(|val| Request {
                            action: RequestAction::AddObject,
                            body: val.clone(),
                        })
                        .collect_vec(),
                };
                let request = serde_json::to_value(request).context("Failed to convert to json")?;
                for url in urls.iter() {
                    let result = HttpClient::post_json::<ImportResponse>(
                        url.clone(),
                        self.headers.inner(),
                        request.clone(),
                    )
                    .await;
                    match result {
                        Ok(_) => {
                            break;
                        }
                        Err(err) => {
                            println!("{err:?}");
                            warn!("failed to get response {err:?}");
                            errors.push(err);
                        }
                    }
                }
            }

            if !errors.is_empty() {
                error!(
                    "Errors from index {:?} during import are {:?}",
                    &index, errors
                );

                bail!("Failed to import")
            }

            Ok(())
        }
    }
}
