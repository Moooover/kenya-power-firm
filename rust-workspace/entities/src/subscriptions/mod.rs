pub mod details;
pub mod plans;

use crate::subscriptions::plans::Plan;
use uuid::Uuid;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct SubscriberId(Uuid);

impl SubscriberId {
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for SubscriberId {
    fn from(value: Uuid) -> Self {
        SubscriberId(value)
    }
}

pub struct Subscriber {
    id: SubscriberId,
    current_plan: Option<Plan>,
}

#[derive(Clone)]
pub enum AffectedSubscriber {
    DirectlyAffected(SubscriberId),
    PotentiallyAffected(SubscriberId),
}

impl AffectedSubscriber {
    pub fn id(&self) -> SubscriberId {
        match self {
            AffectedSubscriber::DirectlyAffected(subscriber) => subscriber.clone(),
            AffectedSubscriber::PotentiallyAffected(subscriber) => subscriber.clone(),
        }
    }
}
