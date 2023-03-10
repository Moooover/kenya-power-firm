use crate::{searcher::Searcher, status_code::StatusCode};
use anyhow::Context;
use async_trait::async_trait;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use shared_kernel::http_client::HttpClient;
use url::Url;
use use_cases::search_for_locations::{LocationApiResponse, LocationSearchApi};

#[derive(Deserialize, Serialize)]
pub struct MatchedSubstrings {
    length: usize,
    offset: usize,
}

#[derive(Deserialize, Serialize)]
pub struct LocationSearchApiResponsePrediction {
    description: String,
    matched_substrings: Vec<MatchedSubstrings>,
    place_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct LocationSearchApiResponse {
    status: StatusCode,
    predictions: Vec<LocationSearchApiResponsePrediction>,
    error_message: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct StatusContainer {
    status: StatusCode,
}

impl LocationSearchApiResponse {
    fn is_cacheable(&self) -> bool {
        self.status.is_cacheable()
    }
}

#[async_trait]
pub trait LocationSearchApiResponseCache: Send + Sync {
    async fn get(&self, key: &Url) -> anyhow::Result<Option<LocationSearchApiResponse>>;
    async fn set(&self, key: &Url, response: &serde_json::Value) -> anyhow::Result<()>;
}

impl From<LocationSearchApiResponse> for Vec<LocationApiResponse> {
    fn from(api_response: LocationSearchApiResponse) -> Self {
        api_response
            .predictions
            .iter()
            .map(|prediction| LocationApiResponse {
                id: prediction.place_id.clone().into(),
                name: prediction.description.clone(),
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl LocationSearchApi for Searcher {
    async fn search(&self, text: String) -> anyhow::Result<Vec<LocationApiResponse>> {
        let search_path = "/place/queryautocomplete/json";
        let url = Url::parse_with_params(
            &format!("{}{}", self.host(), search_path),
            &[("key", self.api_key().expose_secret()), ("input", &text)],
        )
        .context("Failed to parse url")?;

        let cached_response = self.cache.get(&url).await;

        if let Err(err) = &cached_response {
            // TODO: Log error
            println!("{err:?}")
        }

        let response = cached_response.ok().flatten();

        if let Some(response) = response {
            return anyhow::Ok(response.into());
        }

        let api_response = HttpClient::get_json::<serde_json::Value>(url.clone()).await?;
        let response = serde_json::from_value::<LocationSearchApiResponse>(api_response.clone())
            .context("Failed to get valid api response")?;
        if response.is_cacheable() {
            let cached_result = self.cache.set(&url, &api_response).await;
            if let Err(err) = cached_result {
                // TODO: Log error
                println!("{err:?}")
            }
        }

        anyhow::Ok(response.into())
    }
}
