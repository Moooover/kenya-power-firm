use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub email: String,
    pub external_id: String,
}

#[async_trait]
pub trait UserAuthenticationRepo: Send + Sync {
    async fn create_or_update_user(&self, user: User) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AuthenticationInteractor: Send + Sync {
    async fn authenticate(&self, user: User) -> anyhow::Result<()>;
}

pub struct AuthenticationInteractorImpl {
    repo: Arc<dyn UserAuthenticationRepo>,
}

#[async_trait]
impl AuthenticationInteractor for AuthenticationInteractorImpl {
    async fn authenticate(&self, user: User) -> anyhow::Result<()> {
        self.repo.create_or_update_user(user).await
    }
}

impl AuthenticationInteractorImpl {
    pub fn new(repo: Arc<dyn UserAuthenticationRepo>) -> Self {
        Self { repo }
    }
}
