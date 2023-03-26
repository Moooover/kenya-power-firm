use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::Json;
use url::Url;

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
    pub description: String,
    pub place_id: String,
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

impl Repository {
    pub async fn get_cached_text_search_response(
        &self,
        key: &Url,
    ) -> anyhow::Result<Option<LocationSearchApiResponse>> {
        struct Row {
            value: Json<LocationSearchApiResponse>,
        }
        let result = sqlx::query_as!(
            Row,
            r#"
            SELECT value as "value: Json<LocationSearchApiResponse>" FROM location.location_search_cache WHERE key = $1
            "#,
            key.as_str()
        )
        .fetch_optional(self.pool())
        .await
        .context("Failed to fetch from cache")?;

        if let Some(data) = result {
            return Ok(Some(data.value.0));
        }

        Ok(None)
    }

    pub async fn set_cached_text_search_response(
        &self,
        key: &Url,
        response: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let _ = sqlx::query!(
            r#"
            INSERT INTO location.location_search_cache ( key, value )
            VALUES ( $1, $2 ) ON CONFLICT (key)
            DO UPDATE SET value = EXCLUDED.value
            "#,
            key.as_str(),
            Json(response) as _
        )
        .execute(self.pool())
        .await
        .context("Failed to save response in cache")?;

        Ok(())
    }
}
