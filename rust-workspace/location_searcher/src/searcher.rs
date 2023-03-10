use std::sync::Arc;

use anyhow::{Context, Ok};
use use_cases::search_for_locations::{LocationApiResponse, LocationSearchApi};

use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::http_client::HttpClient;
use url::Url;

use crate::configuration::{LocationSearcherConfig, Settings};
use crate::location_details_finder::LocationDetailsCache;
use crate::text_search::LocationSearchApiResponseCache;

#[derive(Clone)]
pub struct Searcher {
    pub(crate) cache: Arc<dyn Cache>,
    config: LocationSearcherConfig,
}

pub trait Cache: LocationSearchApiResponseCache + LocationDetailsCache {}

impl<T> Cache for T where T: LocationDetailsCache + LocationSearchApiResponseCache {}

impl Searcher {
    pub fn new(cache: Arc<dyn Cache>) -> anyhow::Result<Self> {
        let settings = Settings::parse()?;

        Ok(Searcher {
            cache,
            config: settings.location_searcher,
        })
    }

    pub(crate) fn host(&self) -> &str {
        &self.config.host
    }

    pub(crate) fn api_key(&self) -> &Secret<String> {
        &self.config.api_key
    }
}
