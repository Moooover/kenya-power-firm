use std::sync::Arc;

use crate::{
    actor::Actor, authentication::subscriber_authentication::SubscriberResolverInteractor,
};
use async_trait::async_trait;
use entities::{locations::LocationId, subscriptions::SubscriberId};

#[async_trait]
pub trait DeleteLocationsSubscribedToInteractor {
    async fn delete_primary_location(
        &self,
        actor: &dyn Actor,
        location_id: LocationId,
    ) -> anyhow::Result<()>;
}



pub struct DeleteLocationsSubscribedToImpl {
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    delete_op: Arc<dyn DeleteSubscribedLocationOp>
}

impl DeleteLocationsSubscribedToImpl {
    pub fn new(
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
        delete_op: Arc<dyn DeleteSubscribedLocationOp>
    ) -> Self {
        Self {
            subscriber_resolver,
            delete_op
        }
    }
}


#[async_trait]
pub trait DeleteSubscribedLocationOp: Send + Sync {
    async fn delete_subscribed(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()>;
}


#[async_trait]
impl DeleteLocationsSubscribedToInteractor for DeleteLocationsSubscribedToImpl {
    async fn delete_primary_location(
        &self,
        actor: &dyn Actor,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        let subscriber_id = self.subscriber_resolver.resolve_from_actor(actor).await?;
        self.delete_op.delete_subscribed(subscriber_id, location_id).await
    }
}
