use authentication::subscriber_authentication::SubscriberResolverInteractorImpl;
use search_for_locations::{
    LocationSearchApi, LocationSearchInteractor, LocationSearchInteractorImpl,
};
use subscriber_locations::subscribe_to_location::{
    LocationSubscriber, SubscribeToLocationImpl, SubscribeToLocationInteractor,
};

use crate::authentication::{AuthenticationInteractor, AuthenticationInteractorImpl};
use crate::repositories::Repository;
use std::sync::Arc;

pub mod actor;
pub mod authentication;
pub mod import_affected_areas;
pub mod notifications;
mod repositories;
pub mod search_for_locations;
pub mod subscriber_locations;

pub trait App {
    fn authentication(&self) -> &dyn AuthenticationInteractor;
    fn location_searcher(&self) -> &dyn LocationSearchInteractor;
    fn subscribe_to_location(&self) -> &dyn SubscribeToLocationInteractor;
}

pub struct AppImpl {
    authentication: Arc<dyn AuthenticationInteractor>,
    location_searcher_interactor: Arc<dyn LocationSearchInteractor>,
    subscribe_to_location_interactor: Arc<dyn SubscribeToLocationInteractor>,
}

impl App for AppImpl {
    fn authentication(&self) -> &dyn AuthenticationInteractor {
        self.authentication.as_ref()
    }

    fn location_searcher(&self) -> &dyn LocationSearchInteractor {
        self.location_searcher_interactor.as_ref()
    }

    fn subscribe_to_location(&self) -> &dyn SubscribeToLocationInteractor {
        self.subscribe_to_location_interactor.as_ref()
    }
}

pub trait LocationsApi: LocationSearchApi + LocationSubscriber {}

impl<T> LocationsApi for T where T: LocationSearchApi + LocationSubscriber {}

impl AppImpl {
    pub fn new<R: Repository + 'static, L: LocationsApi + 'static>(
        repo: R,
        location_api: L,
    ) -> Self {
        let repository = Arc::new(repo);
        let location_api = Arc::new(location_api);
        let subscriber_authentication_checker =
            Arc::new(SubscriberResolverInteractorImpl::new(repository.clone()));
        let authentication_interactor = AuthenticationInteractorImpl::new(repository);
        let location_searcher_interactor = LocationSearchInteractorImpl::new(
            location_api.clone(),
            subscriber_authentication_checker.clone(),
        );

        let subscribe_to_locations_interactor =
            SubscribeToLocationImpl::new(subscriber_authentication_checker, location_api);

        Self {
            authentication: Arc::new(authentication_interactor),
            location_searcher_interactor: Arc::new(location_searcher_interactor),
            subscribe_to_location_interactor: Arc::new(subscribe_to_locations_interactor),
        }
    }
}
