mod db_access;

use crate::contracts::list_subscribed_locations::db_access::SubscribedLocationsAccess;
use crate::contracts::LocationSubscriptionSubSystem;
use crate::data_transfer::LocationDetails;
use shared_kernel::subscriber_id::SubscriberId;

impl LocationSubscriptionSubSystem {
    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn list_subscribed_locations(
        &self,
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<Vec<LocationDetails>> {
        let db = SubscribedLocationsAccess::new();
        db.get_subscribed_locations(subscriber_id).await
    }
}
