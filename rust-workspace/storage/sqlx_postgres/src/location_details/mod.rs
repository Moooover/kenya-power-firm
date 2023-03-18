use anyhow::Context;
use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use std::collections::HashMap;
use use_cases::subscriber_locations::data::LocationId;

use crate::repository::Repository;

use crate::locations::insert_location::NonAcronymString;

impl Repository {
    pub async fn find_location_id(
        &self,
        location: ExternalLocationId,
    ) -> anyhow::Result<Option<LocationId>> {
        let pool = self.pool();
        let db_results = sqlx::query!(
            r#"
            SELECT id
            FROM location.locations WHERE external_id = $1
            "#,
            location.inner()
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch location by id")?;

        Ok(db_results.map(|record| record.id.into()))
    }
}
