use crate::authentication::{
    subscriber_authentication::SubscriberResolverRepo, SubscriberAuthenticationRepo,
};

use crate::import_affected_areas::SaveBlackoutAffectedAreasRepo;

pub trait Repository:
    SubscriberAuthenticationRepo + SaveBlackoutAffectedAreasRepo + SubscriberResolverRepo + Clone
{
}

impl<T> Repository for T where
    T: Clone
        + SubscriberAuthenticationRepo
        + SaveBlackoutAffectedAreasRepo
        + SubscriberResolverRepo
{
}
