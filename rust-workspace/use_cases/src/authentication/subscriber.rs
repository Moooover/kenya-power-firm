use crate::actor::Actor;
use async_trait::async_trait;
use std::sync::Arc;
use subscriptions::subscriber::SubscriberId;

#[async_trait]
pub trait SubscriberResolverInteractor: Send + Sync {
    async fn resolve_from_actor(&self, actor: &dyn Actor) -> anyhow::Result<SubscriberId>;
}

#[async_trait]
pub trait SubscriberResolverRepo: Send + Sync {
    async fn find(&self, actor: &dyn Actor) -> anyhow::Result<SubscriberId>;
}

pub struct SubscriberResolverInteractorImpl {
    repo: Arc<dyn SubscriberResolverRepo>,
}

#[async_trait]
impl SubscriberResolverInteractor for SubscriberResolverInteractorImpl {
    async fn resolve_from_actor(&self, actor: &dyn Actor) -> anyhow::Result<SubscriberId> {
        self.repo.find(actor).await
    }
}

impl SubscriberResolverInteractorImpl {
    pub fn new(repo: Arc<dyn SubscriberResolverRepo>) -> Self {
        Self { repo }
    }
}
