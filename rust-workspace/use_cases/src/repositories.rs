use crate::{
    authentication::{
        subscriber_authentication::SubscriberResolverRepo, SubscriberAuthenticationRepo,
    },
    subscriber_locations::{
        list_subscribed_locations::LocationsSubscribedRepo,
        subscribe_to_location::SubscribeToLocationRepo,
    },
};
use crate::{
    import_affected_areas::SaveBlackoutAffectedAreasRepo,
    notifications::notify_subscribers::SubscriberRepo,
};

pub trait Repository:
    SubscriberAuthenticationRepo
    + SaveBlackoutAffectedAreasRepo
    + SubscriberResolverRepo
    + SubscribeToLocationRepo
    + SubscriberRepo
    + LocationsSubscribedRepo
    + Clone
{
}

impl<T> Repository for T where
    T: Clone
        + SubscriberAuthenticationRepo
        + SaveBlackoutAffectedAreasRepo
        + SubscriberResolverRepo
        + SubscribeToLocationRepo
        + LocationsSubscribedRepo
        + SubscriberRepo
{
}
