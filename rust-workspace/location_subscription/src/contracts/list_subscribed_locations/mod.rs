mod db_access;

use crate::contracts::list_subscribed_locations::db_access::SubscribedLocationsAccess;
use crate::data_transfer::LocationDetails;
use shared_kernel::subscriber_id::SubscriberId;

pub struct ListSubscribedLocationsInteractor;

impl ListSubscribedLocationsInteractor {
    #[tracing::instrument(err, level = "info")]
    pub async fn list_subscribed_locations(
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<Vec<LocationDetails>> {
        let db = SubscribedLocationsAccess::new();
        db.get_subscribed_locations(subscriber_id).await
    }
}
