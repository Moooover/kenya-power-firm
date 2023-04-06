pub mod details;

use serde::{Deserialize, Serialize};

use shared_kernel::uuid_key;

uuid_key!(SubscriberId);

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
