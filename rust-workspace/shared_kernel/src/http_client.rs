use anyhow::{Context, Error};
use bytes::Bytes;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Response;
use reqwest_tracing::TracingMiddleware;
use std::collections::HashMap;
use thiserror::Error as ThisError;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

lazy_static! {
    static ref CLIENT: ClientWithMiddleware =   {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(reqwest::Client::new())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .with(TracingMiddleware::default())
        .build()
    };
}

pub struct HttpClient;

#[derive(ThisError, Debug)]
pub enum HttpClientError {
    #[error(transparent)]
    ResponseError(#[from] Error),
    #[error("httpBuilderError {0}")]
    HTTPBuilderError(String),
}


struct HeadersMapGenerator(HeaderMap);


impl HeadersMapGenerator {
    fn into_inner(self) -> HeaderMap {
        self.0
    }
}


impl TryFrom<HashMap<&'static str, String>> for HeadersMapGenerator {
    type Error = HttpClientError;

    fn try_from(value: HashMap<&'static str, String>) -> Result<Self, Self::Error> {
        let mut header_map = HeaderMap::new();

        for (key, value) in value.into_iter() {
            let value = HeaderValue::from_str(&value)
                .map_err(|err| HttpClientError::HTTPBuilderError(format!("{err} {value}")))?;
            header_map.insert(key, value);
        }
        Ok(Self(header_map))
    }
}

impl HttpClient {
    async fn get(url: Url) -> anyhow::Result<Response> {
        CLIENT
            .get(url.clone())
            .send()
            .await
            .with_context(|| format!("Failed to fetch request from {url}"))
    }
    pub async fn get_bytes(url: Url) -> anyhow::Result<Bytes> {
        Self::get(url.clone())
            .await?
            .bytes()
            .await
            .context("Failed to get bytes response")
    }
    pub async fn get_text(url: Url) -> anyhow::Result<String> {
        Self::get(url.clone())
            .await?
            .text()
            .await
            .context("Failed to get text response")
    }

    pub async fn get_json<DTO: DeserializeOwned>(url: Url) -> anyhow::Result<DTO> {
        let response = Self::get(url.clone()).await?;
        let err_msg = format!("Failed to deserialize response {response:?}");
        response.json::<DTO>().await.context(err_msg)
    }

    pub async fn get_with_headers<DTO: DeserializeOwned>(
        url: Url,
        headers: HashMap<&'static str, String>,
    ) -> Result<DTO, HttpClientError> {
        let generator = HeadersMapGenerator::try_from(headers)?;
        let header_map = generator.into_inner();
        CLIENT
            .get(url)
            .headers(header_map)
            .send()
            .await
            .context("Failed to get json response")
            .map_err(HttpClientError::ResponseError)?
            .json::<DTO>()
            .await
            .context("Failed to deserialize response")
            .map_err(HttpClientError::ResponseError)
    }

    pub async fn post_json<DTO: DeserializeOwned>(
        url: Url,
        headers: HashMap<&'static str, String>,
        body: Value,
    ) -> Result<DTO, HttpClientError> {
        let generator = HeadersMapGenerator::try_from(headers)?;
        let header_map = generator.into_inner();
        CLIENT
            .post(url)
            .headers(header_map)
            .json(&body)
            .send()
            .await
            .context("Failed to get json response")
            .map_err(HttpClientError::ResponseError)?
            .json::<DTO>()
            .await
            .context("Failed to deserialize response")
            .map_err(HttpClientError::ResponseError)
    }
}
