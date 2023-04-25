use crate::{
    authentication::{
        subscriber_authentication::SubscriberResolverRepo, SubscriberAuthenticationRepo,
    },
    subscriber_locations::{
        delete_locations_subscribed_to::DeleteSubscribedLocationsRepo,
        list_subscribed_locations::LocationsSubscribedRepo,
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
        + LocationsSubscribedRepo
        + DeleteSubscribedLocationsRepo
        + SubscriberRepo
{
}
