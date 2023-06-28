use crate::actor::Actor;
use async_trait::async_trait;
use entities::contracts::SubscriberContracts;
use shared_kernel::subscriber_id::SubscriberId;
use std::sync::Arc;

#[async_trait]
pub trait SubscriberResolverInteractor: Send + Sync {
    async fn resolve_from_actor(&self, actor: &dyn Actor) -> anyhow::Result<SubscriberId>;
}

pub struct SubscriberResolverInteractorImpl {}

#[async_trait]
impl SubscriberResolverInteractor for SubscriberResolverInteractorImpl {
    async fn resolve_from_actor(&self, actor: &dyn Actor) -> anyhow::Result<SubscriberId> {
        let external_id = actor.external_id().into_inner();
        SubscriberContracts::authenticate(external_id).await
    }
}

impl SubscriberResolverInteractorImpl {
    pub fn new() -> Self {
        Self {}
    }
}
