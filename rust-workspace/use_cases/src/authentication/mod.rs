pub mod subscriber;

use crate::actor::{Actor, ExternalId};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
pub struct User {
    pub details: UserDetails,
    pub external_id: ExternalId,
}

#[derive(Debug)]
pub struct UserDetails {
    pub name: String,
    pub email: String,
}

#[async_trait]
pub trait UserAuthenticationRepo: Send + Sync {
    async fn create_or_update_user(&self, user: User) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AuthenticationInteractor: Send + Sync {
    async fn authenticate(&self, actor: &dyn Actor, user: UserDetails) -> anyhow::Result<()>;
}

pub struct AuthenticationInteractorImpl {
    repo: Arc<dyn UserAuthenticationRepo>,
}

#[async_trait]
impl AuthenticationInteractor for AuthenticationInteractorImpl {
    async fn authenticate(&self, actor: &dyn Actor, user: UserDetails) -> anyhow::Result<()> {
        let user = User {
            details: user,
            external_id: actor.external_id(),
        };
        self.repo.create_or_update_user(user).await
    }
}

impl AuthenticationInteractorImpl {
    pub fn new(repo: Arc<dyn UserAuthenticationRepo>) -> Self {
        Self { repo }
    }
}
