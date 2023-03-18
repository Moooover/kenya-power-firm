pub mod details;
pub mod plans;

use crate::subscriptions::plans::Plan;
use uuid::Uuid;

use shared_kernel::uuid_key;

uuid_key!(SubscriberId);

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
