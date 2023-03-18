use anyhow::Context;
use celery::{prelude::TaskError, task::TaskResult};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use shared_kernel::http_client::HttpClient;
use sqlx_postgres::cache::location_search::LocationSearchApiResponse;
use sqlx_postgres::cache::location_search::StatusCode;
use url::Url;

use crate::{
    callbacks::failure_callback,
    configuration::{REPO, SETTINGS_CONFIG},
};

pub fn generate_search_url(text: String) -> anyhow::Result<Url> {
    let search_path = "/place/queryautocomplete/json";

    let host = &SETTINGS_CONFIG.host;

    Url::parse_with_params(
        &format!("{}{}", host, search_path),
        &[
            ("key", SETTINGS_CONFIG.api_key.expose_secret()),
            ("input", &text),
        ],
    )
    .context("Failed to parse url")
}

#[celery::task(max_retries = 200, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn search_locations_by_text(text: String) -> TaskResult<()> {
    let url =
        generate_search_url(text).map_err(|err| TaskError::UnexpectedError(format!("{err}")))?;

    let repo = REPO.get().await;
    let cached_response = repo
        .get_cached_text_search_response(&url)
        .await
        .map_err(|err| TaskError::UnexpectedError(format!("{err}")))?;
    if cached_response.is_some() {
        // Don't return anything, once the client makes a second request
        // the response will be in the cache ready for them
        return Ok(());
    }

    let api_response = HttpClient::get_json::<serde_json::Value>(url.clone())
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))?;

    let response = serde_json::from_value::<LocationSearchApiResponse>(api_response.clone())
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))?;
    if response.is_cacheable() {
        return repo
            .set_cached_text_search_response(&url, &api_response)
            .await
            .map_err(|err| TaskError::UnexpectedError(format!("{err}")));
    }

    Err(TaskError::UnexpectedError(format!(
        "Invalid status code in the response {response:?}"
    )))
}
