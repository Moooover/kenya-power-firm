use crate::save_and_search_for_locations::search_engine;

pub struct ImportLocationsToSearchEngine;

impl ImportLocationsToSearchEngine {
    pub async fn execute() -> anyhow::Result<()> {
        search_engine::import_primary_locations::execute().await?;
        search_engine::import_nearby_locations::execute().await
    }
}
