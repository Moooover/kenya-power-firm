use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use use_cases::notifications::notify_subscribers::{
    GetPreferredDeliveryStrategies, StrategyWithSubscribers,
};

use crate::{notifications::email::EmailStrategy, producer::Producer};

#[async_trait]
impl GetPreferredDeliveryStrategies for Producer {
    async fn get_strategies(
        &self,
        subscribers: Vec<SubscriberId>,
    ) -> anyhow::Result<Vec<StrategyWithSubscribers>> {
        let email_strategy = EmailStrategy::new_strategy(self.app.clone());
        // for now, its just email notifications
        Ok(vec![StrategyWithSubscribers {
            strategy: email_strategy,
            subscribers,
        }])
    }
}
