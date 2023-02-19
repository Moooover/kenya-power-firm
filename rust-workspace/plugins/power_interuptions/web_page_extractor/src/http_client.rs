use anyhow::{Context, Error};
use bytes::Bytes;
use lazy_static::lazy_static;
use reqwest::Response;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use url::Url;

lazy_static! {
    static ref CLIENT: ClientWithMiddleware =   {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(reqwest::Client::new())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
    };
}

pub struct HttpClient;

impl HttpClient {
    async fn get(url: Url) -> anyhow::Result<Response> {
        CLIENT
            .get(url.clone())
            .send()
            .await
            .with_context(|| format!("Failed to fetch request from {url}"))
    }
    pub async fn get_bytes(url: Url) -> Result<Bytes, Error> {
        Self::get(url.clone())
            .await?
            .bytes()
            .await
            .with_context(|| format!("Failed to get bytes from {url}"))
    }
    pub async fn get_text(url: Url) -> anyhow::Result<String> {
        Self::get(url.clone())
            .await?
            .text()
            .await
            .with_context(|| format!("Failed to get text from {url}"))
    }
}
