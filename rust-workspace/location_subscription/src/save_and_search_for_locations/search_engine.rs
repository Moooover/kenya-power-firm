use anyhow::{Context, Ok};
use entities::locations::LocationId;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use self::algolia_search_engine::AlgoliaClient;

pub struct SearchEngine {
    client: AlgoliaClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEngineLocationInput {
    pub id: LocationId,
    pub name: String,
    pub address: String,
    pub api_response: Value,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            client: AlgoliaClient::new(),
        }
    }

    pub(super) async fn save_object<DTO: DeserializeOwned>(
        &self,
        index: impl ToString,
        body: Value,
    ) -> anyhow::Result<DTO> {
        self.client.post::<DTO>(index.to_string(), body).await
    }

    pub(super) async fn search<DTO: DeserializeOwned>(
        &self,
        index: impl ToString,
        query: impl ToString,
    ) -> anyhow::Result<DTO> {
        self.client.get(index.to_string(), query.to_string()).await
    }
}

mod algolia_search_engine {
    use anyhow::{anyhow, bail, Context};

    use itertools::Itertools;
    use serde::de::DeserializeOwned;
    use serde_json::Value;
    use shared_kernel::non_empty_string;
    use shared_kernel::{http_client::HttpClient, string_key};
    use std::{collections::HashMap, iter};
    use tracing::{error, warn};
    use url::Url;

    use crate::config::SETTINGS_CONFIG;
    string_key!(APIKey);
    string_key!(ApplicationId);
    non_empty_string!(IndexName);
    non_empty_string!(Query);

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
        client: HttpClient,
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
                client: HttpClient,
                hosts: Hosts::new(application_id),
                headers,
            }
        }

        pub async fn post<DTO: DeserializeOwned>(
            &self,
            index: impl TryInto<IndexName, Error = String>,
            body: Value,
        ) -> Result<DTO, anyhow::Error> {
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
                    HttpClient::post_json::<DTO>(url, self.headers.inner(), Some(body.clone()))
                        .await;
                match result {
                    Ok(res) => return Ok(res),
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

        pub async fn get<DTO: DeserializeOwned>(
            &self,
            index: impl TryInto<IndexName, Error = String>,
            query: impl TryInto<Query, Error = String>,
        ) -> Result<DTO, anyhow::Error> {
            let index = index.try_into().map_err(|err| anyhow!(err))?;
            let query = query.try_into().map_err(|err| anyhow!(err))?;

            let urls = self
                .hosts
                .read_hosts
                .iter()
                .map(|host| {
                    Url::parse_with_params(
                        &format!("https://{}/1/indexes/{}/query", host, index),
                        &[("query", query.clone())],
                    )
                    .context("Failed to parse url")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;
            let mut errors = vec![];
            for url in urls {
                let result = HttpClient::post_json::<DTO>(url, self.headers.inner(), None).await;
                match result {
                    Ok(res) => return Ok(res),
                    Err(err) => {
                        warn!("failed to get response {err:?}");
                        errors.push(err);
                    }
                }
            }
            error!(
                "Errors from index {} & query {} are {:?}",
                &index, &query, errors
            );
            bail!("Failed to return response from algolia")
        }
    }
}
