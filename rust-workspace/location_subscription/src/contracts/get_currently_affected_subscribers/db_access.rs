
use crate::save_and_search_for_locations::{AffectedLocation, SaveAndSearchLocations};


pub struct GetCurrentlyAffectedLocations {
    db: SaveAndSearchLocations,
}

impl GetCurrentlyAffectedLocations {
    pub fn new() -> Self {
        Self {
            db: SaveAndSearchLocations::new(),
        }
    }
    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn currently_affected_locations(&self) -> anyhow::Result<Vec<AffectedLocation>> {
        self.db.currently_affected_locations().await
    }
}
