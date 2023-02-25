use crate::authentication::UserAuthenticationRepo;

pub trait Repository: UserAuthenticationRepo + Clone {}

impl<T> Repository for T where T: Clone + UserAuthenticationRepo {}
