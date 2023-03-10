use crate::repository::Repository;
use async_trait::async_trait;
use entities::power_interruptions::location::{AffectedLine, Region};
use entities::subscriptions::AffectedSubscriber;
use std::collections::HashMap;
use use_cases::notifications::notify_subscribers::SubscriberRepo;

#[async_trait]
impl SubscriberRepo for Repository {
    async fn get_affected_subscribers(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<HashMap<AffectedSubscriber, Vec<AffectedLine>>> {
        todo!()
    }
}
