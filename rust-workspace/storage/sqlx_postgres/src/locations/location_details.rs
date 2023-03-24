use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use entities::locations::LocationId;
use std::collections::HashMap;

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
