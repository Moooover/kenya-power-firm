mod db_access;

use crate::contracts::get_affected_subscribers_from_import::AffectedSubscribersInteractor;
use crate::data_transfer::{AffectedSubscriber, LocationMatchedAndLineSchedule};
use std::collections::HashMap;

pub struct CurrentlyAffectedSubscribersInteractor;

impl Default for CurrentlyAffectedSubscribersInteractor {
    fn default() -> Self {
        Self::new()
    }
}

impl CurrentlyAffectedSubscribersInteractor {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get(
        &self,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<LocationMatchedAndLineSchedule>>> {
        let db = db_access::GetCurrentlyAffectedLocations::new();
        let locations = db.currently_affected_locations().await?;
        AffectedSubscribersInteractor::affected_subscribers_from_locations(locations).await
    }
}
