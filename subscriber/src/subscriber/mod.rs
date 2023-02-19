use crate::plans::Plan;
use uuid::Uuid;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct SubscriberId(Uuid);

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
