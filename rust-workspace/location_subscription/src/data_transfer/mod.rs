use entities::locations::LocationId;
use entities::power_interruptions::location::{LocationName, NairobiTZDateTime};
use entities::subscriptions::SubscriberId;
use serde::{Deserialize, Serialize};
use shared_kernel::uuid_key;
use url::Url;

uuid_key!(LineScheduleId);

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LocationMatchedAndLineSchedule {
    pub line_schedule: LineWithScheduledInterruptionTime,
    pub location_id: LocationId,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AffectedSubscriberWithLocationMatchedAndLineSchedule {
    pub affected_subscriber: AffectedSubscriber,
    pub location_matched: LocationMatchedAndLineSchedule,
}

#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
pub struct LineWithScheduledInterruptionTime {
    pub line_name: String,
    pub from: NairobiTZDateTime,
    pub to: NairobiTZDateTime,
    pub source_url: Url,
}

pub struct LocationDetails {
    pub id: LocationId,
    pub name: LocationName,
    pub address: String,
}
