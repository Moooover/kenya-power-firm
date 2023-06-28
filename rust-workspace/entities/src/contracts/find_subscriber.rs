use crate::contracts::SubscriberContracts;
use crate::find_subscriber::FindSubscriber;
use crate::subscriptions::details::{SubscriberDetails, SubscriberExternalId};
use shared_kernel::subscriber_id::SubscriberId;

impl SubscriberContracts {
    #[tracing::instrument(err, level = "info")]
    pub async fn find_by_subscriber_id(
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<SubscriberDetails> {
        let subscriber_finder = FindSubscriber::new();
        subscriber_finder.find_subscriber_by_id(subscriber_id).await
    }
}
