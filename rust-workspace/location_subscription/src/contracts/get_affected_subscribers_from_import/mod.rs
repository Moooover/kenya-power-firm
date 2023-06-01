mod db_access;

use crate::contracts::get_affected_subscribers_from_import::db_access::AffectedSubscribersDbAccess;
use crate::data_transfer::{AffectedSubscriber, LocationMatchedAndLineSchedule};
use crate::save_and_search_for_locations::AffectedLocation;
use entities::power_interruptions::location::FutureOrCurrentNairobiTZDateTime;
use std::collections::HashMap;
use url::Url;

pub struct AffectedSubscribersInteractor;

#[derive(Debug, Clone)]
pub struct TimeFrame {
    pub from: FutureOrCurrentNairobiTZDateTime,
    pub to: FutureOrCurrentNairobiTZDateTime,
}

#[derive(Debug)]
pub struct Area {
    pub name: String,
    pub time_frame: TimeFrame,
    pub locations: Vec<String>,
}

#[derive(Debug)]
pub struct County {
    pub name: String,
    pub areas: Vec<Area>,
}
#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub counties: Vec<County>,
}

#[derive(Debug)]
pub struct ImportInput(pub HashMap<Url, Vec<Region>>);

impl AffectedSubscribersInteractor {
    #[tracing::instrument(err, level = "info")]
    pub async fn get_affected_subscribers_from_import(
        input: ImportInput,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<LocationMatchedAndLineSchedule>>> {
        let db = AffectedSubscribersDbAccess::new();
        let mut result = HashMap::new();
        for (url, regions) in input.0.into_iter() {
            result.extend(
                db.get_affected_subscribers(url, &regions)
                    .await?
                    .into_iter(),
            )
        }

        Ok(result)
    }

    #[tracing::instrument(err, level = "info")]
    pub(crate) async fn affected_subscribers_from_locations(
        affected_locations: Vec<AffectedLocation>,
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<LocationMatchedAndLineSchedule>>> {
        let db = AffectedSubscribersDbAccess::new();
        db.affected_subscribers_from_affected_locations(affected_locations)
            .await
    }
}
