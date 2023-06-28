pub mod subscriber_authentication;

use crate::actor::Actor;
use async_trait::async_trait;
use subscribers::contracts::create_or_update_subscriber::SubscriberInput;
use subscribers::contracts::SubscriberContracts;

#[derive(Debug)]
pub struct SubscriberDetailsInput {
    pub name: String,
    pub email: String,
}

#[async_trait]
pub trait AuthenticationInteractor: Send + Sync {
    async fn authenticate(
        &self,
        actor: &dyn Actor,
        user: SubscriberDetailsInput,
    ) -> anyhow::Result<()>;
}

pub struct AuthenticationInteractorImpl {}

#[async_trait]
impl AuthenticationInteractor for AuthenticationInteractorImpl {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn authenticate(
        &self,
        actor: &dyn Actor,
        subscriber: SubscriberDetailsInput,
    ) -> anyhow::Result<()> {
        let external_id = actor.external_id().into_inner();

        SubscriberContracts::create_or_update_subscriber(SubscriberInput {
            name: subscriber.name,
            email: subscriber.email,
            external_id: external_id.to_string(),
        })
        .await
    }
}

impl AuthenticationInteractorImpl {
    pub fn new() -> Self {
        Self {}
    }
}
