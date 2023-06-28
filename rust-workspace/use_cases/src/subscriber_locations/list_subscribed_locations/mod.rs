use crate::actor::Actor;
use crate::authentication::subscriber_authentication::SubscriberResolverInteractor;
use async_trait::async_trait;
use shared_kernel::location_ids::LocationId;
use shared_kernel::subscriber_id::SubscriberId;
use std::sync::Arc;

#[async_trait]
pub trait ListSubscribedLocationsInteractor: Send + Sync {
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<LocationWithId>>;
}

#[async_trait]
pub trait ListSubscribedLocationsOp: Send + Sync {
    async fn list(&self, id: SubscriberId) -> anyhow::Result<Vec<LocationWithId>>;
}

pub struct ListSubscribedLocationsImpl {
    list_op: Arc<dyn ListSubscribedLocationsOp>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
}

impl ListSubscribedLocationsImpl {
    pub fn new(
        list_op: Arc<dyn ListSubscribedLocationsOp>,
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    ) -> Self {
        Self {
            list_op,
            subscriber_resolver,
        }
    }
}

#[async_trait]
impl ListSubscribedLocationsInteractor for ListSubscribedLocationsImpl {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn list(&self, actor: &dyn Actor) -> anyhow::Result<Vec<LocationWithId>> {
        let id = self.subscriber_resolver.resolve_from_actor(actor).await?;

        self.list_op.list(id).await
    }
}

pub struct LocationWithId {
    pub id: LocationId,
    pub name: String,
    pub address: String,
}
