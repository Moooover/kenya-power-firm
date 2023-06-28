mod db_access;

use shared_kernel::location_ids::LocationId;
use shared_kernel::subscriber_id::SubscriberId;

pub struct UnsubscribeFromLocationInteractor;

impl UnsubscribeFromLocationInteractor {
    #[tracing::instrument(err, level = "info")]
    pub async fn unsubscribe_from_location(
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        db_access::UnsubscribeDbAccess::new()
            .unsubscribe(subscriber_id, location_id)
            .await
    }
}
