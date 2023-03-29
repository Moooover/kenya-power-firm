use std::sync::Arc;

use async_trait::async_trait;
use entities::locations::ExternalLocationId;
use uuid::Uuid;

use crate::{
    actor::Actor, authentication::subscriber_authentication::SubscriberResolverInteractor,
};

#[derive(Copy, Clone)]
pub enum Status {
    Pending,
    Success,
    Failure,
    NotFound,
}

pub struct LocationApiResponse {
    pub id: ExternalLocationId,
    pub name: String,
    pub address: String,
}

pub struct LocationResponseWithStatus {
    pub responses: Vec<LocationApiResponse>,
    pub status: Status,
}

#[async_trait]
pub trait LocationSearchApi: Send + Sync {
    async fn search(&self, text: String) -> anyhow::Result<LocationResponseWithStatus>;
}

#[async_trait]
pub trait LocationSearchInteractor {
    async fn search(
        &self,
        actor: &dyn Actor,
        text: String,
    ) -> anyhow::Result<LocationResponseWithStatus>;
}

pub struct LocationSearchInteractorImpl {
    search_api: Arc<dyn LocationSearchApi>,
    subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
}

impl LocationSearchInteractorImpl {
    pub fn new(
        search_api: Arc<dyn LocationSearchApi>,
        subscriber_resolver: Arc<dyn SubscriberResolverInteractor>,
    ) -> Self {
        Self {
            search_api,
            subscriber_resolver,
        }
    }
}

#[async_trait]
impl LocationSearchInteractor for LocationSearchInteractorImpl {
    async fn search(
        &self,
        actor: &dyn Actor,
        text: String,
    ) -> anyhow::Result<LocationResponseWithStatus> {
        // you just need to be authenticated in order to do the search
        let _ = self.subscriber_resolver.resolve_from_actor(actor).await?;
        self.search_api.search(text).await
    }
}
