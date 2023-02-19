use crate::actor::Actor;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use subscriptions::subscriber::SubscriberId;
use uuid::Uuid;

#[derive(Copy, Clone)]
pub struct StrategyId(Uuid);

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
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<Strategy>>;
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
}

#[async_trait]
impl ListAllNotificationStrategiesInteractor for ListAllNotificationsStrategiesImpl {
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<StrategyWithIsActive>> {
        let all_strategies = self.repo.strategies().await?;

        let subscriber_strategies = self.repo.subscriber_strategies(actor.id()).await?;

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
