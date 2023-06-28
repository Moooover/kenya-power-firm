use authentication::subscriber_authentication::SubscriberResolverInteractorImpl;

use search_for_locations::{
    LocationSearchApi, LocationSearchInteractor, LocationSearchInteractorImpl,
};
use subscriber_locations::{
    delete_locations_subscribed_to::{
        DeleteLocationsSubscribedToImpl, DeleteLocationsSubscribedToInteractor,
    },
    list_subscribed_locations::{ListSubscribedLocationsImpl, ListSubscribedLocationsInteractor},
    subscribe_to_location::{
        LocationSubscriber, SubscribeToLocationImpl, SubscribeToLocationInteractor,
    },
};

use crate::authentication::{AuthenticationInteractor, AuthenticationInteractorImpl};
use crate::subscriber_locations::delete_locations_subscribed_to::DeleteSubscribedLocationOp;
use crate::subscriber_locations::list_subscribed_locations::ListSubscribedLocationsOp;
use std::sync::Arc;

pub mod actor;
pub mod authentication;
pub mod search_for_locations;
pub mod subscriber_locations;

pub trait App {
    fn authentication(&self) -> &dyn AuthenticationInteractor;
    fn location_searcher(&self) -> &dyn LocationSearchInteractor;
    fn subscribe_to_location(&self) -> &dyn SubscribeToLocationInteractor;
    fn list_locations_subcribed_to(&self) -> &dyn ListSubscribedLocationsInteractor;
    fn delete_subscribed_location(&self) -> &dyn DeleteLocationsSubscribedToInteractor;
}

pub struct AppImpl {
    authentication: Arc<dyn AuthenticationInteractor>,
    location_searcher_interactor: Arc<dyn LocationSearchInteractor>,
    locations_subscribed_to_interactor: Arc<dyn ListSubscribedLocationsInteractor>,
    subscribe_to_location_interactor: Arc<dyn SubscribeToLocationInteractor>,
    delete_subscribed_location: Arc<dyn DeleteLocationsSubscribedToInteractor>,
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

    fn list_locations_subcribed_to(&self) -> &dyn ListSubscribedLocationsInteractor {
        self.locations_subscribed_to_interactor.as_ref()
    }

    fn delete_subscribed_location(&self) -> &dyn DeleteLocationsSubscribedToInteractor {
        self.delete_subscribed_location.as_ref()
    }
}

pub trait LocationsApi: LocationSearchApi + LocationSubscriber {}
impl<T> LocationsApi for T where T: LocationSearchApi + LocationSubscriber {}

pub trait LocationSubscriptionOperations:
    DeleteSubscribedLocationOp + ListSubscribedLocationsOp
{
}
impl<T> LocationSubscriptionOperations for T where
    T: DeleteSubscribedLocationOp + ListSubscribedLocationsOp
{
}

impl AppImpl {
    pub fn new<L: LocationsApi + 'static>(
        location_api: L,
        location_subscription_operations: impl LocationSubscriptionOperations + 'static,
    ) -> Self {
        let location_api = Arc::new(location_api);
        let subscriber_authentication_checker = Arc::new(SubscriberResolverInteractorImpl::new());
        let authentication_interactor = AuthenticationInteractorImpl::new();
        let location_searcher_interactor = LocationSearchInteractorImpl::new(
            location_api.clone(),
            subscriber_authentication_checker.clone(),
        );

        let subscribe_to_locations_interactor =
            SubscribeToLocationImpl::new(subscriber_authentication_checker.clone(), location_api);

        let location_subscription = Arc::new(location_subscription_operations);
        let locations_subscribed_to_interactor = ListSubscribedLocationsImpl::new(
            location_subscription.clone(),
            subscriber_authentication_checker.clone(),
        );

        let delete_subscribed_locations = DeleteLocationsSubscribedToImpl::new(
            subscriber_authentication_checker,
            location_subscription,
        );

        Self {
            authentication: Arc::new(authentication_interactor),
            locations_subscribed_to_interactor: Arc::new(locations_subscribed_to_interactor),
            location_searcher_interactor: Arc::new(location_searcher_interactor),
            subscribe_to_location_interactor: Arc::new(subscribe_to_locations_interactor),
            delete_subscribed_location: Arc::new(delete_subscribed_locations),
        }
    }
}
