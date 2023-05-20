pub mod subscriber_authentication;

use crate::actor::Actor;
use anyhow::anyhow;
use async_trait::async_trait;
use entities::subscriptions::details::SubscriberDetails;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;

#[derive(Debug)]
pub struct SubscriberDetailsInput {
    pub name: String,
    pub email: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubscriberAuthenticationRepo: Send + Sync {
    async fn create_or_update_subscriber(
        &self,
        subscriber: SubscriberDetails,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AuthenticationInteractor: Send + Sync {
    async fn authenticate(
        &self,
        actor: &dyn Actor,
        user: SubscriberDetailsInput,
    ) -> anyhow::Result<()>;
}

pub struct AuthenticationInteractorImpl {
    repo: Arc<dyn SubscriberAuthenticationRepo>,
}

#[async_trait]
impl AuthenticationInteractor for AuthenticationInteractorImpl {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn authenticate(
        &self,
        actor: &dyn Actor,
        subscriber: SubscriberDetailsInput,
    ) -> anyhow::Result<()> {
        let external_id = actor.external_id().into();

        let details = SubscriberDetails {
            name: subscriber.name.try_into().map_err(|err| anyhow!("{err}"))?,
            email: subscriber
                .email
                .try_into()
                .map_err(|err| anyhow!("{err}"))?,
            external_id,
        };

        self.repo.create_or_update_subscriber(details).await
    }
}

impl AuthenticationInteractorImpl {
    pub fn new(repo: Arc<dyn SubscriberAuthenticationRepo>) -> Self {
        Self { repo }
    }
}

#[cfg(test)]
mod tests {

    use crate::actor::{MockActor, SubscriberExternalId};
    use crate::authentication::MockSubscriberAuthenticationRepo;
    use crate::authentication::{
        AuthenticationInteractor, AuthenticationInteractorImpl, SubscriberDetailsInput,
    };

    use std::sync::Arc;

    fn mock_actor() -> MockActor {
        let mut mock_actor = MockActor::new();
        mock_actor
            .expect_external_id()
            .returning(|| SubscriberExternalId::try_from("auth|external_id".to_string()).unwrap());
        mock_actor
    }

    #[tokio::test]
    async fn test_that_invalid_email_is_not_submitted() {
        let mock_repo = MockSubscriberAuthenticationRepo::new();
        let mock_actor = mock_actor();
        let interactor = AuthenticationInteractorImpl::new(Arc::new(mock_repo));

        let result = interactor
            .authenticate(
                &mock_actor,
                SubscriberDetailsInput {
                    name: "Tevin".to_string(),
                    email: "just-an-email.com".to_string(),
                },
            )
            .await;
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_subscriber_is_not_authenticated_with_empty_name() {
        let mock_repo = MockSubscriberAuthenticationRepo::new();
        let mock_actor = mock_actor();

        let interactor = AuthenticationInteractorImpl::new(Arc::new(mock_repo));

        let result = interactor
            .authenticate(
                &mock_actor,
                SubscriberDetailsInput {
                    name: "".to_string(),
                    email: "blackouts.dev@gmail.com".to_string(),
                },
            )
            .await;
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_subscriber_is_authenticated_if_details_are_valid() {
        let mut mock_repo = MockSubscriberAuthenticationRepo::new();
        mock_repo
            .expect_create_or_update_subscriber()
            .returning(|_| Ok(()));
        let mock_actor = mock_actor();

        let interactor = AuthenticationInteractorImpl::new(Arc::new(mock_repo));

        let result = interactor
            .authenticate(
                &mock_actor,
                SubscriberDetailsInput {
                    name: "Blackouts Subscriber".to_string(),
                    email: "blackouts.dev@gmail.com".to_string(),
                },
            )
            .await;
        assert!(result.is_ok())
    }
}
