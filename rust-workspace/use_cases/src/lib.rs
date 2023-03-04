use crate::authentication::{AuthenticationInteractor, AuthenticationInteractorImpl};
use crate::repositories::Repository;
use std::sync::Arc;

pub mod actor;
pub mod authentication;
pub mod import_affected_areas;
pub mod locations;
pub mod notifications;
mod repositories;

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
    pub fn new<R: Repository + 'static>(repo: R) -> Self {
        let repository = Arc::new(repo);
        let authentication_interactor = AuthenticationInteractorImpl::new(repository.clone());

        Self {
            authentication: Arc::new(authentication_interactor),
        }
    }
}
