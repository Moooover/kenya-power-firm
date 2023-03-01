use crate::authentication::SubscriberAuthenticationRepo;

pub trait Repository: SubscriberAuthenticationRepo + Clone {}

impl<T> Repository for T where T: Clone + SubscriberAuthenticationRepo {}
