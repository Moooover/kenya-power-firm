use crate::authentication::SubscriberAuthenticationRepo;
use crate::import_affected_areas::SaveBlackoutAffectedAreasRepo;

pub trait Repository: SubscriberAuthenticationRepo + SaveBlackoutAffectedAreasRepo + Clone {}

impl<T> Repository for T where
    T: Clone + SubscriberAuthenticationRepo + SaveBlackoutAffectedAreasRepo
{
}
