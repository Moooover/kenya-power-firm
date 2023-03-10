use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    actor::Actor, authentication::subscriber_authentication::SubscriberResolverInteractor,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExternalLocationId(String);

impl AsRef<str> for ExternalLocationId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ExternalLocationId {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for ExternalLocationId {
    fn from(value: String) -> Self {
        ExternalLocationId(value)
    }
}
pub struct LocationApiResponse {
    pub id: ExternalLocationId,
    pub name: String,
}

#[async_trait]
pub trait LocationSearchApi: Send + Sync {
    async fn search(&self, text: String) -> anyhow::Result<Vec<LocationApiResponse>>;
}

#[async_trait]
pub trait LocationSearchInteractor {
    async fn search(
        &self,
        actor: &dyn Actor,
        text: String,
    ) -> anyhow::Result<Vec<LocationApiResponse>>;
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
    ) -> anyhow::Result<Vec<LocationApiResponse>> {
        // you just need to be authenticated in order to do the search
        let _ = self.subscriber_resolver.resolve_from_actor(actor).await?;
        self.search_api.search(text).await
    }
}
