use std::collections::HashMap;

use crate::repository::Repository;
use anyhow::Context;
use entities::locations::LocationInput;
use sqlx::types::Json;

pub struct NonAcronymString(String);

impl NonAcronymString {
    pub fn into_inner(self) -> String {
        self.0
    }
}

// TODO: Fix this logic
impl From<String> for NonAcronymString {
    fn from(value: String) -> Self {
        let acronym_map = HashMap::from([
            ("pri", "Primary"),
            ("rd", "Road"),
            ("est", "Estate"),
            ("sch", "School"),
            ("sec", "Secondary"),
            ("stn", "Station"),
            ("apts", "Apartment"),
            ("hqtrs", "Headquaters"),
            ("mkt", "Market"),
        ]);
        let split = value
            .split(' ')
            .map(|val| {
                format!(
                    "{} ",
                    acronym_map
                        .get(val.to_ascii_lowercase().as_str())
                        .cloned()
                        .unwrap_or(val)
                )
            })
            .collect::<String>()
            .trim()
            .to_owned();
        NonAcronymString(split)
    }
}

impl AsRef<str> for NonAcronymString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Repository {
    pub async fn insert_location(&self, location: LocationInput) -> anyhow::Result<()> {
        // locations
        let pool = self.pool();
        let external_id = location.external_id.as_ref();
        let address = location.address.clone();
        let sanitized_address = NonAcronymString::from(location.address);

        sqlx::query!(
            "
            INSERT INTO location.locations (name, external_id, address, sanitized_address, external_api_response) 
            VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING
            ",
            location.name,
            external_id,
            address,
            sanitized_address.as_ref(),
            Json(location.api_response) as _
        )
        .execute(pool)
        .await
        .context("Failed to insert location")?;

        Ok(())
    }
}
