use crate::contracts::SubscribersSubsystem;
use crate::find_subscriber::FindSubscriber;

use shared_kernel::subscriber_id::SubscriberId;

pub use crate::find_subscriber::SubscriberDetails;
pub use crate::find_subscriber::SubscriberExternalId;

impl SubscribersSubsystem {
    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn find_by_subscriber_id(
        &self,
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<SubscriberDetails> {
        let subscriber_finder = FindSubscriber::new();
        subscriber_finder.find_subscriber_by_id(subscriber_id).await
    }
}
