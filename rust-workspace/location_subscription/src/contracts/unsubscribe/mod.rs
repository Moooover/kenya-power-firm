mod db_access;

use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;

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
