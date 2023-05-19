use crate::contracts::text_search::db_access::TextSearchDbAccess;
use shared_kernel::{string_key};

mod db_access;

pub struct TextSearcher;

impl TextSearcher {
    pub fn new() -> Self {
        TextSearcher
    }

    pub async fn api_search(&self, text: String) -> anyhow::Result<Vec<LocationDetails>> {
        let db = TextSearchDbAccess::new();
        search::api_search(text, &db).await
    }

    pub async fn cache_search(&self, text: String) -> anyhow::Result<Option<Vec<LocationDetails>>> {
        let db = TextSearchDbAccess::new();
        search::cache_search(text, &db).await
    }
}

string_key!(ExternalLocationId);

pub struct LocationDetails {
    pub id: ExternalLocationId,
    pub name: String,
    pub address: String,
}

pub(crate) mod search {
    use crate::config::SETTINGS_CONFIG;
    use crate::contracts::text_search::db_access::TextSearchDbAccess;
    use crate::contracts::text_search::{LocationDetails};
    use anyhow::{anyhow, Context};
    use secrecy::ExposeSecret;
    use serde::Deserialize;
    use serde::Serialize;
    use shared_kernel::http_client::HttpClient;
    use shared_kernel::non_empty_string;
    use url::Url;
    non_empty_string!(LocationSearchText);

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum StatusCode {
        OK,
        #[serde(rename = "ZERO_RESULTS")]
        ZERORESULTS,
        #[serde(rename = "INVALID_REQUEST")]
        INVALIDREQUEST,
        #[serde(rename = "OVER_QUERY_LIMIT")]
        OVERQUERYLIMIT,
        #[serde(rename = "REQUEST_DENIED")]
        REQUESTDENIED,
        #[serde(rename = "UNKNOWN_ERROR")]
        UNKNOWNERROR,
    }

    impl StatusCode {
        pub fn is_cacheable(&self) -> bool {
            matches!(self, StatusCode::OK | StatusCode::ZERORESULTS)
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct LocationSearchApiResponsePrediction {
        description: String,
        place_id: Option<String>,
        /// At this point, we don't need to fully define the structure of the values
        /// so that we can save everything from the response.
        matched_substrings: serde_json::Value,
        structured_formatting: serde_json::Value,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct LocationSearchApiResponse {
        status: StatusCode,
        pub predictions: Vec<LocationSearchApiResponsePrediction>,
        error_message: Option<String>,
    }

    impl LocationSearchApiResponse {
        pub fn is_cacheable(&self) -> bool {
            self.status.is_cacheable()
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub(crate) struct ValidResponse(LocationSearchApiResponse);

    impl ValidResponse {
        fn new(response: LocationSearchApiResponse) -> anyhow::Result<Self> {
            if response.is_cacheable() {
                let predictions = ValidResponse::remove_invalid_predictions(response.predictions);
                Ok(ValidResponse(LocationSearchApiResponse {
                    predictions,
                    ..response
                }))
            } else {
                Err(anyhow::anyhow!("Invalid response"))
            }
        }

        fn remove_invalid_predictions(
            predictions: Vec<LocationSearchApiResponsePrediction>,
        ) -> Vec<LocationSearchApiResponsePrediction> {
            predictions
                .into_iter()
                .filter(|prediction| prediction.place_id.is_some())
                .collect()
        }
    }

    pub fn generate_search_url(text: String) -> anyhow::Result<Url> {
        let path_details = "/place/autocomplete/json";
        let host_with_path = &format!("{}{}", SETTINGS_CONFIG.location.host, path_details);
        Url::parse_with_params(
            host_with_path,
            &[
                ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
                ("input", &text),
                ("components", &"country:ke".to_string()),
            ],
        )
        .context("Failed to parse url")
    }

    pub(crate) async fn api_search(
        text: impl TryInto<LocationSearchText, Error = String>,
        db: &TextSearchDbAccess,
    ) -> anyhow::Result<Vec<LocationDetails>> {
        let text = text
            .try_into()
            .map_err(|err| anyhow!("Cannot search for location with empty text. Error: {}", err))?;
        let url = generate_search_url(text.inner())?;
        let cached_response = db.get_cached_text_search_response(&url).await?;
        if let Some(response) = cached_response {
            return Ok(response);
        }
        let api_response = HttpClient::get_json::<LocationSearchApiResponse>(url.clone()).await?;

        let valid_response = ValidResponse::new(api_response)?;

        db.set_cached_text_search_response(&url, valid_response)
            .await?;

        db.get_cached_text_search_response(&url)
            .await?
            .ok_or_else(|| anyhow!("Should have cached response for url: {}", url))
    }

    pub(crate) async fn cache_search(
        text: impl TryInto<LocationSearchText, Error = String>,
        db: &TextSearchDbAccess,
    ) -> anyhow::Result<Option<Vec<LocationDetails>>> {
        let text = text
            .try_into()
            .map_err(|err| anyhow!("Cannot search for location with empty text. Error: {}", err))?;
        let url = generate_search_url(text.inner())?;
        db.get_cached_text_search_response(&url).await
    }
}
