use crate::authentication::AuthenticationInteractor;
use std::sync::Arc;

pub mod actor;
pub mod authentication;
pub mod import_planned_blackouts;
pub mod locations;
pub mod notifications;

pub trait App {
    fn authentication(&self) -> &dyn AuthenticationInteractor;
}

pub struct AppImpl {
    authentication: Arc<dyn AuthenticationInteractor>,
}

impl App for AppImpl {
    fn authentication(&self) -> &dyn AuthenticationInteractor {
        self.authentication.as_ref()
    }
}

impl AppImpl {
    pub fn new(authentication: Arc<dyn AuthenticationInteractor>) -> Self {
        Self { authentication }
    }
}
