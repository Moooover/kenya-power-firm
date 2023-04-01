use crate::{
    authentication::{
        subscriber_authentication::SubscriberResolverRepo, SubscriberAuthenticationRepo,
    },
    subscriber_locations::{
        list_subscribed_locations::LocationsSubscribedRepo,
        subscribe_to_location::SubscribeToLocationRepo, delete_locations_subscribed_to::DeleteSubscribedLocationsRepo,
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
    + DeleteSubscribedLocationsRepo
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
        + DeleteSubscribedLocationsRepo
        + SubscriberRepo
{
}
