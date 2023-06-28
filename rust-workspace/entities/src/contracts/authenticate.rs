use crate::contracts::SubscriberContracts;
use crate::find_subscriber::FindSubscriber;
use crate::subscriptions::details::SubscriberExternalId;
use shared_kernel::subscriber_id::SubscriberId;
use std::fmt::Debug;

impl SubscriberContracts {
    #[tracing::instrument(err, level = "info")]
    pub async fn authenticate(
        external_id: impl AsRef<str> + Debug,
    ) -> anyhow::Result<SubscriberId> {
        let external_id = SubscriberExternalId::try_from(external_id.as_ref().to_owned())
            .map_err(|err| anyhow::anyhow!(err))?;
        let subscriber_finder = FindSubscriber::new();
        subscriber_finder.find_by_external_id(external_id).await
    }
}
