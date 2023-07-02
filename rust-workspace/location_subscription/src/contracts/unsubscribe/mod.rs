mod db_access;

use crate::contracts::LocationSubscriptionSubSystem;
use shared_kernel::location_ids::LocationId;
use shared_kernel::subscriber_id::SubscriberId;

impl LocationSubscriptionSubSystem {
    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn unsubscribe_from_location(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        db_access::UnsubscribeDbAccess::new()
            .unsubscribe(subscriber_id, location_id)
            .await
    }
}
