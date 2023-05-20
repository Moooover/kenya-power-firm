use crate::contracts::text_search::search::{StatusCode, ValidResponse};
use crate::contracts::text_search::LocationDetails;
use crate::db_access::DbAccess;
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::Json;
use url::Url;

pub struct TextSearchDbAccess {
    db: DbAccess,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StructuredFormatting {
    pub main_text: String,
    pub secondary_text: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationSearchApiResponsePrediction {
    pub description: String,
    pub place_id: String,
    pub structured_formatting: StructuredFormatting,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocationSearchApiResponse {
    status: StatusCode,
    pub predictions: Vec<LocationSearchApiResponsePrediction>,
    error_message: Option<String>,
}

impl TextSearchDbAccess {
    pub fn new() -> Self {
        TextSearchDbAccess { db: DbAccess }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub(crate) async fn get_cached_text_search_response(
        &self,
        key: &Url,
    ) -> anyhow::Result<Option<Vec<LocationDetails>>> {
        let pool = self.db.pool().await;
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
        .fetch_optional(pool.as_ref())
        .await
        .context("Failed to fetch from cache")?;

        if let Some(data) = result {
            let data = data.value.0;
            return Ok(Some(
                data.predictions
                    .into_iter()
                    .map(|data| LocationDetails {
                        id: data.place_id.into(),
                        name: data.structured_formatting.main_text,
                        address: data.structured_formatting.secondary_text,
                    })
                    .collect(),
            ));
        }

        Ok(None)
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub(crate) async fn set_cached_text_search_response(
        &self,
        key: &Url,
        response: ValidResponse,
    ) -> anyhow::Result<()> {
        let api_response =
            serde_json::to_string(&response).context("Failed to convert api_response to string")?;

        let api_response_as_json: serde_json::Value = serde_json::from_str(&api_response)
            .with_context(|| {
                format!("Failed to convert api_response to JSON value {api_response}")
            })?;

        let pool = self.db.pool().await;
        let _ = sqlx::query!(
            r#"
            INSERT INTO location.location_search_cache ( key, value )
            VALUES ( $1, $2 ) ON CONFLICT (key)
            DO UPDATE SET value = EXCLUDED.value
            "#,
            key.as_str(),
            Json(api_response_as_json) as _
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to save response in cache")?;

        Ok(())
    }
}
