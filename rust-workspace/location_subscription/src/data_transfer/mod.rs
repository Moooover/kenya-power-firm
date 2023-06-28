use serde::{Deserialize, Serialize};
use shared_kernel::date_time::nairobi_date_time::NairobiTZDateTime;
use shared_kernel::location_ids::LocationId;
use shared_kernel::subscriber_id::SubscriberId;
use shared_kernel::{string_key, uuid_key};
use url::Url;

uuid_key!(LineScheduleId);
string_key!(LocationName);

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LocationMatchedAndLineSchedule {
    pub line_schedule: LineWithScheduledInterruptionTime,
    pub location_id: LocationId,
    pub location_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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
