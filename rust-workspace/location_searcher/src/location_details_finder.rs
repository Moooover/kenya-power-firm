use crate::{searcher::Searcher, status_code::StatusCode};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use secrecy::ExposeSecret;
use serde::Deserialize;
use shared_kernel::http_client::HttpClient;
use std::collections::{HashMap, HashSet};
use url::Url;
use use_cases::search_for_locations::ExternalLocationId;
use use_cases::subscriber_locations::data::{LocationId, LocationInput};
use use_cases::subscriber_locations::subscribe_to_location::LocationDetailsFinder;

pub struct LocationDetailsInput {
    pub id: ExternalLocationId,
    pub name: String,
    pub address: String,
    pub api_response: serde_json::Value, // TODO: Clean this up
}

#[async_trait]
pub trait LocationDetailsCache: Send + Sync {
    async fn find_many(
        &self,
        locations: Vec<ExternalLocationId>,
    ) -> anyhow::Result<HashMap<ExternalLocationId, LocationId>>;

    async fn save_many(&self, locations: Vec<LocationDetailsInput>) -> anyhow::Result<()>;
}

const PLACE_DETAILS_PATH: &str = "/place/details/json";

#[async_trait]
impl LocationDetailsFinder for Searcher {
    async fn location_details(
        &self,
        location: LocationInput<ExternalLocationId>,
    ) -> anyhow::Result<LocationInput<LocationId>> {
        let all_ids = location.ids();

        let found = self.cache.find_many(all_ids.clone()).await?;

        let found_keys: HashSet<_> = HashSet::from_iter(found.keys().cloned());
        let all_ids_hash_set = HashSet::from_iter(location.ids().into_iter());

        let ids_not_found_in_cache = all_ids_hash_set.difference(&found_keys).collect::<Vec<_>>();

        if ids_not_found_in_cache.is_empty() {
            return LocationInputAndCacheResultsWrapper(location, found).try_into();
        }

        let urls = ids_not_found_in_cache
            .into_iter()
            .map(|id| {
                Url::parse_with_params(
                    &format!("{}{}", &self.host(), PLACE_DETAILS_PATH),
                    &[
                        ("key", self.api_key().expose_secret()),
                        ("place_id", &id.as_ref().to_owned()),
                    ],
                )
                .context("Failed to parse url")
            })
            .collect::<Result<Vec<_>, _>>()?;

        let url_count = urls.len();

        let mut futures: FuturesUnordered<_> = urls.into_iter().map(get_place_details).collect();

        let mut errors = vec![];
        let mut details = Vec::with_capacity(url_count);
        while let Some(result) = futures.next().await {
            match result {
                Ok(data) => details.push(data),
                Err(error) => errors.push(error),
            }
        }

        if !errors.is_empty() {
            return Err(anyhow!("{errors:?}"));
        }

        self.cache.save_many(details).await?;

        let mapping_of_external_id_to_location_id = self
            .cache
            .find_many(all_ids_hash_set.into_iter().collect())
            .await?;

        LocationInputAndCacheResultsWrapper(location, mapping_of_external_id_to_location_id)
            .try_into()
    }
}

struct LocationInputAndCacheResultsWrapper(
    LocationInput<ExternalLocationId>,
    HashMap<ExternalLocationId, LocationId>,
);

impl TryFrom<LocationInputAndCacheResultsWrapper> for LocationInput<LocationId> {
    type Error = anyhow::Error;

    fn try_from(value: LocationInputAndCacheResultsWrapper) -> Result<Self, Self::Error> {
        let primary_id = value.0.primary_id();
        let primary_id = value.1.get(primary_id).ok_or(anyhow!(
            "Failed to find id for location with external identifier {primary_id:?}"
        ))?;
        let nearby_location_ids = value
            .0
            .nearby_locations
            .iter()
            .map(|location| {
                value.1.get(location).cloned().ok_or(anyhow!(
                    "Failed to find id for location with external identifier {primary_id:?}"
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LocationInput {
            id: *primary_id,
            nearby_locations: nearby_location_ids,
        })
    }
}

async fn get_place_details(url: Url) -> anyhow::Result<LocationDetailsInput> {
    #[derive(Deserialize, Debug, Clone)]

    struct ResponseResult {
        name: String,
        formatted_address: String,
        place_id: String,
    }
    #[derive(Deserialize, Debug, Clone)]

    struct Response {
        result: ResponseResult,
        status: StatusCode,
    }
    let result = HttpClient::get_json::<serde_json::Value>(url).await?;
    let response: Response =
        serde_json::from_value(result.clone()).context("Failed to get valid response")?;
    if response.status.is_cacheable() {
        return Ok(LocationDetailsInput {
            id: response.result.place_id.into(),
            name: response.result.name,
            address: response.result.formatted_address,
            api_response: result,
        });
    }
    Err(anyhow!("Failed to get valid response {result:?}"))
}
