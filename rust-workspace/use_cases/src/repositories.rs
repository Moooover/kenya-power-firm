use crate::authentication::{
    subscriber_authentication::SubscriberResolverRepo, SubscriberAuthenticationRepo,
};

pub trait Repository: SubscriberAuthenticationRepo + SubscriberResolverRepo + Clone {}

impl<T> Repository for T where T: Clone + SubscriberAuthenticationRepo + SubscriberResolverRepo {}
