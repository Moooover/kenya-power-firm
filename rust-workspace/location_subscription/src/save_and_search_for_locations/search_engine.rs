use anyhow::Context;
use entities::locations::LocationId;
use serde::de::DeserializeOwned;
use serde_json::Value;
use shared_kernel::non_empty_string;
use shared_kernel::{
    http_client::{HttpClient, HttpClientError},
    string_key,
};
use std::collections::HashMap;
use url::Url;

use crate::config::{Settings, SETTINGS_CONFIG};

pub struct AlgoliaSearchEngine {
    client: AlgoliaClient,
}

pub struct SearchEngineLocationInput {
    pub id: LocationId,
    pub name: String,
    pub address: String,
    pub api_response: Value,
}

impl AlgoliaSearchEngine {
    pub fn new() -> Self {
        Self {
            client: AlgoliaClient::new(),
        }
    }

    pub(super) async fn save_object(&self, input: SearchEngineLocationInput) {}

    pub(super) async fn search(&self, query: String) -> Vec<LocationId> {
        vec![]
    }
}

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
        Self {
            read_hosts: vec![],
            write_hosts: vec![],
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

struct AlgoliaClient {
    client: HttpClient,
    hosts: Hosts,
    headers: AlgoliaHeaders,
}

impl AlgoliaClient {
    pub fn new() -> Self {
        let api_key = SETTINGS_CONFIG.search_engine.api_key.clone().into();
        let application_id: ApplicationId =
            SETTINGS_CONFIG.search_engine.application_key.clone().into();
        let headers = AlgoliaHeaders::new(api_key, application_id);
        Self {
            client: HttpClient,
            hosts: Hosts::new(application_id.clone()),
            headers,
        }
    }

    pub async fn post<DTO: DeserializeOwned>(
        url: Url,
        index: IndexName,
        body: Value,
    ) -> Result<DTO, anyhow::Error> {
        todo!()
    }

    pub async fn get<DTO: DeserializeOwned>(
        &self,
        index: IndexName,
        query: Query,
    ) -> Result<DTO, anyhow::Error> {
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
        for url in urls {
            let result = HttpClient::get_with_headers::<DTO>(url, self.headers.inner()).await;
            
        }
        todo!()
    }
}
