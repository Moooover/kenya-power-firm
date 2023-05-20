use crate::actor::Actor;
use crate::authentication::subscriber_authentication::SubscriberResolverInteractor;
use async_trait::async_trait;
use entities::notifications::strategy::StrategyId;
use entities::subscriptions::SubscriberId;
use std::collections::HashSet;
use std::sync::Arc;

pub struct Strategy {
    pub id: StrategyId,
    pub name: String,
}

pub struct StrategyWithIsActive {
    pub strategy: Strategy,
    pub is_active: bool,
}

#[async_trait]
pub trait ListAllNotificationStrategiesInteractor: Send + Sync {
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<StrategyWithIsActive>>;
}

#[async_trait]
pub trait StrategiesRepo: Send + Sync {
    async fn strategies(&self) -> anyhow::Result<Vec<Strategy>>;
}

#[async_trait]
pub trait SubscriberStrategyRepo: Send + Sync {
    async fn subscriber_strategies(
        &self,
        subscriber: SubscriberId,
    ) -> anyhow::Result<HashSet<StrategyId>>;
}

pub trait Strategies: SubscriberStrategyRepo + StrategiesRepo {}

pub struct ListAllNotificationsStrategiesImpl {
    repo: Arc<dyn Strategies>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
}

impl ListAllNotificationsStrategiesImpl {
    pub fn new(
        repo: Arc<dyn Strategies>,
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    ) -> Self {
        Self {
            repo,
            subscriber_resolver,
        }
    }
}

#[async_trait]
impl ListAllNotificationStrategiesInteractor for ListAllNotificationsStrategiesImpl {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<StrategyWithIsActive>> {
        let all_strategies = self.repo.strategies().await?;
        let id = self.subscriber_resolver.resolve_from_actor(actor).await?;
        let subscriber_strategies = self.repo.subscriber_strategies(id).await?;

        Ok(all_strategies
            .into_iter()
            .map(|strategy| {
                let id = strategy.id;
                StrategyWithIsActive {
                    strategy,
                    is_active: subscriber_strategies.contains(&id),
                }
            })
            .collect())
    }
}
