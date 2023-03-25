pub mod details;
pub mod plans;

use crate::subscriptions::plans::Plan;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::locations::LocationId;
use shared_kernel::uuid_key;

uuid_key!(SubscriberId);

pub struct Subscriber {
    id: SubscriberId,
    current_plan: Option<Plan>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

impl AffectedSubscriber {
    pub fn id(&self) -> SubscriberId {
        match self {
            AffectedSubscriber::DirectlyAffected(subscriber) => *subscriber,
            AffectedSubscriber::PotentiallyAffected(subscriber) => *subscriber,
        }
    }
}
