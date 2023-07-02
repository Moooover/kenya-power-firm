use background_workers::producer::Producer;
use location_subscription::contracts::LocationSubscriptionSubSystem;
use subscribers::contracts::SubscribersSubsystem;

pub struct Application {
    pub location_subscription: LocationSubscriptionSubSystem,
    pub producer: Producer,
    pub subscribers: SubscribersSubsystem,
}

impl Application {
    pub fn new(producer: Producer) -> Self {
        Application {
            location_subscription: LocationSubscriptionSubSystem {},
            producer,
            subscribers: SubscribersSubsystem {},
        }
    }
}
