use anyhow::Context;
use async_trait::async_trait;
use location_searcher::location_details_finder::{LocationDetailsCache, LocationDetailsInput};
use std::collections::HashMap;
use use_cases::search_for_locations::ExternalLocationId;
use use_cases::subscriber_locations::data::LocationId;

use crate::repository::Repository;

use crate::locations::insert_location::NonAcronymString;

#[async_trait]
impl LocationDetailsCache for Repository {
    async fn find_many(
        &self,
        locations: Vec<ExternalLocationId>,
    ) -> anyhow::Result<HashMap<ExternalLocationId, LocationId>> {
        let locations = locations
            .into_iter()
            .map(|location| location.as_ref().to_owned())
            .collect::<Vec<_>>();
        let pool = self.pool();
        let db_results = sqlx::query!(
            r#"
            SELECT id, external_id
            FROM location.locations WHERE external_id = ANY($1)
            "#,
            &locations[..]
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch locations by external_ids")?;

        Ok(db_results
            .into_iter()
            .map(|record| (record.external_id.into(), record.id.into()))
            .collect())
    }

    async fn save_many(&self, locations: Vec<LocationDetailsInput>) -> anyhow::Result<()> {
        let number_of_locations = locations.len();
        let mut names = Vec::with_capacity(number_of_locations);
        let mut external_ids = Vec::with_capacity(number_of_locations);
        let mut addresses = Vec::with_capacity(number_of_locations);
        let mut sanitized_address = Vec::with_capacity(number_of_locations);
        let mut external_api_responses = Vec::with_capacity(number_of_locations);

        for location in locations.into_iter() {
            names.push(location.name);
            external_ids.push(location.id.as_ref().to_owned());
            addresses.push(location.address.clone());
            sanitized_address.push(NonAcronymString::from(location.address).into_inner());
            external_api_responses.push(location.api_response);
        }
        sqlx::query!(
            "
            INSERT INTO location.locations (name, external_id, address, sanitized_address, external_api_response) 
            SELECT * FROM UNNEST ($1::text[], $2::text[], $3::text[], $4::text[], $5::jsonb[]) ON CONFLICT DO NOTHING
            ",
            &names[..],
            &external_ids[..],
            &addresses[..],
            &sanitized_address[..],
            &external_api_responses[..]
        ).execute(self.pool()).await.context("Failed to insert locations")?;

        Ok(())
    }
}
