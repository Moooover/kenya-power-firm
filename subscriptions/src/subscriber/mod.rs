use crate::plans::Plan;

pub struct SubscriberId(String);

pub struct Subscriber {
    id: SubscriberId,
    current_plan: Option<Plan>,
}
