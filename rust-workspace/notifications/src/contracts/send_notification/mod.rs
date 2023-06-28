pub(crate) mod db_access;
pub mod email;

use serde::{Deserialize, Serialize};
use shared_kernel::date_time::nairobi_date_time::NairobiTZDateTime;
use shared_kernel::location_ids::LocationId;
use shared_kernel::subscriber_id::SubscriberId;
use url::Url;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

impl AffectedSubscriber {
    pub fn id(&self) -> SubscriberId {
        match self {
            Self::DirectlyAffected(subscriber_id) => *subscriber_id,
            Self::PotentiallyAffected(subscriber_id) => *subscriber_id,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct LineWithScheduledInterruptionTime {
    pub line_name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Location {
    pub location_id: LocationId,
    pub name: String,
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct LocationMatchedAndLineSchedule {
    pub line_schedule: LineWithScheduledInterruptionTime,
    pub location: Location,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AffectedSubscriberWithLocations {
    pub source_url: Url,
    pub subscriber: AffectedSubscriber,
    pub locations: Vec<LocationMatchedAndLineSchedule>,
}
