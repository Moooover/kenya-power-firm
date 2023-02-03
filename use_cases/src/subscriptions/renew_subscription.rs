use async_trait::async_trait;
use std::error::Error;
use subscription::plans::Plan;
use subscription::SubscriberId;

#[async_trait]
pub trait RenewSubscriptionInteractor {
    async fn renew(&self, subscriber: SubscriberId, plan: Plan) -> Result<(), Box<dyn Error>>;
}
