use crate::contracts::text_search::db_access::TextSearchDbAccess;
use shared_kernel::string_key;

mod db_access;

pub struct TextSearcher;

impl Default for TextSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSearcher {
    pub fn new() -> Self {
        TextSearcher
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn api_search(&self, text: String) -> anyhow::Result<Vec<LocationDetails>> {
        let db = TextSearchDbAccess::new();
        search::api_search(text, &db).await
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn cache_search(&self, text: String) -> anyhow::Result<Option<Vec<LocationDetails>>> {
        let db = TextSearchDbAccess::new();
        search::cache_search(text, &db).await
    }
}

string_key!(ExternalLocationId);

#[derive(Debug)]
pub struct LocationDetails {
    pub id: ExternalLocationId,
    pub name: String,
    pub address: String,
}

pub(crate) mod search {
    use crate::config::SETTINGS_CONFIG;
    use crate::contracts::text_search::db_access::TextSearchDbAccess;
    use crate::contracts::text_search::LocationDetails;
    use anyhow::{anyhow, Context};
    use secrecy::ExposeSecret;
    use serde::Deserialize;
    use serde::Serialize;
    use shared_kernel::http_client::HttpClient;
    use shared_kernel::non_empty_string;
    use std::fmt::Debug;
    use url::Url;

    #[cfg(test)]
    use httpmock::prelude::*;
    #[cfg(test)]
    use lazy_static::lazy_static;

    #[cfg(test)]
    lazy_static! {
        static ref SERVER: MockServer = MockServer::connect_from_env();
    }

    non_empty_string!(LocationSearchText);

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum StatusCode {
        OK,
        #[serde(rename = "ZERO_RESULTS")]
        ZeroResults,
        #[serde(rename = "INVALID_REQUEST")]
        InvalidRequest,
        #[serde(rename = "OVER_QUERY_LIMIT")]
        OverQueryLimit,
        #[serde(rename = "REQUEST_DENIED")]
        RequestDenied,
        #[serde(rename = "UNKNOWN_ERROR")]
        UnknownError,
    }

    impl StatusCode {
        pub fn is_cacheable(&self) -> bool {
            matches!(self, StatusCode::OK | StatusCode::ZeroResults)
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

    #[tracing::instrument(skip(db), level = "info")]
    pub(crate) async fn api_search<T>(
        text: T,
        db: &TextSearchDbAccess,
    ) -> anyhow::Result<Vec<LocationDetails>>
    where
        T: TryInto<LocationSearchText, Error = String> + Debug,
    {
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

    #[tracing::instrument(skip(db), level = "info")]
    pub(crate) async fn cache_search<T>(
        text: T,
        db: &TextSearchDbAccess,
    ) -> anyhow::Result<Option<Vec<LocationDetails>>>
    where
        T: TryInto<LocationSearchText, Error = String> + Debug,
    {
        let text = text
            .try_into()
            .map_err(|err| anyhow!("Cannot search for location with empty text. Error: {}", err))?;
        let url = generate_search_url(text.inner())?;
        db.get_cached_text_search_response(&url).await
    }

    #[cfg(test)]
    mod tests {
        use std::vec;

        use httpmock::{Method::GET, MockServer};
        use serde_json::json;

        use crate::contracts::text_search::{
            search::{LocationSearchApiResponsePrediction, StatusCode},
            TextSearcher,
        };

        #[tokio::test]
        async fn test_that_searcher_gets_results_from_cache_if_request_was_made_previously() {
            let server = MockServer::connect_from_env();

            let mut mock = server.mock(|when, then| {
                when.method(GET)
                    .query_param("input", "text")
                    .path("/place/autocomplete/json");
                let predictions: Vec<LocationSearchApiResponsePrediction> = vec![];
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(
                        json!({ "status":  StatusCode::ZeroResults, "predictions": predictions  }),
                    );
            });

            let searcher = TextSearcher::new();
            let search_text = "text";
            let result = searcher.api_search(search_text.to_string()).await;
            println!("{result:?}");
            assert!(result.is_ok());
            let result_2 = searcher.api_search(search_text.to_string()).await;
            assert!(result_2.is_ok());
            // only 1 request was made to the api
            mock.assert_hits(1);
            mock.delete();
        }

        #[tokio::test]
        async fn test_an_error_is_thown_if_status_is_not_cacheable() {
            let searcher = TextSearcher::new();

            {
                let server = MockServer::connect_from_env();

                let query_1 = "query1";

                let mut mock_query_1 = server.mock(|when, then| {
                when.method(GET)
                    .query_param("input", query_1)
                    .path("/place/autocomplete/json");
                let predictions: Vec<LocationSearchApiResponsePrediction> = vec![];
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(
                        json!({ "status":  StatusCode::OverQueryLimit, "predictions": predictions  }),
                    );
            });
                let result = searcher.api_search(query_1.to_string()).await;
                assert!(result.is_err());
                mock_query_1.assert();
                mock_query_1.delete();
            }

            {
                let server = MockServer::connect_from_env();
                let query_2 = "query2";

                let mut mock_query_2 = server.mock(|when, then| {
                when.method(GET)
                    .query_param("input", query_2)
                    .path("/place/autocomplete/json");
                let predictions: Vec<LocationSearchApiResponsePrediction> = vec![];
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(
                        json!({ "status":  StatusCode::RequestDenied, "predictions": predictions  }),
                    );
            });
                let result = searcher.api_search(query_2.to_string()).await;
                assert!(result.is_err());
                mock_query_2.assert();
                mock_query_2.delete();
            }

            {
                let server = MockServer::connect_from_env();
                let query_3 = "query3";
                let mut mock_query_3 = server.mock(|when, then| {
                    when.method(GET)
                        .query_param("input", query_3)
                        .path("/place/autocomplete/json");
                    let predictions: Vec<LocationSearchApiResponsePrediction> = vec![];
                    then.status(200)
                    .header("content-type", "application/json")
                    .json_body(
                        json!({ "status":  StatusCode::UnknownError, "predictions": predictions  }),
                    );
                });
                let result = searcher.api_search(query_3.to_string()).await;
                assert!(result.is_err());
                mock_query_3.assert();
                mock_query_3.delete();
            }
        }
    }
}
