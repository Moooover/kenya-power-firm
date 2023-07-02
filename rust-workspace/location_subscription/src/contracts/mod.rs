



#[cfg(feature = "internal_contracts")]
pub mod get_affected_subscribers_from_import;
#[cfg(feature = "internal_contracts")]
pub mod get_currently_affected_subscribers;
#[cfg(feature = "internal_contracts")]
pub mod import_locations_to_search_engine;
#[cfg(feature = "internal_contracts")]
pub mod subscribe;

#[cfg(feature = "contracts")]
pub mod list_subscribed_locations;
#[cfg(feature = "contracts")]
pub mod unsubscribe;

#[derive(Clone)]
pub struct LocationSubscriptionSubSystem;
