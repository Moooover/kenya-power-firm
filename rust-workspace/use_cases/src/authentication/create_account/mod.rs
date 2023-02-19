use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

pub struct NewUser {
    pub name: String,
    pub email: String,
    pub external_id: String,
}

#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("account already exists")]
    AlreadyExists,
    #[error("internal server error")]
    InternalServerError(#[from] anyhow::Error),
}

#[async_trait]
pub trait UserRegistrationRepo: Send + Sync {
    async fn save_user(&self, user: NewUser) -> Result<(), CreateAccountError>;
}

#[async_trait]
pub trait CreateAccountInteractor: Send + Sync {
    async fn create_account(&self, user: NewUser) -> Result<(), CreateAccountError>;
}

pub struct CreateAccountImpl {
    repo: Arc<dyn UserRegistrationRepo>,
}

#[async_trait]
impl CreateAccountInteractor for CreateAccountImpl {
    async fn create_account(&self, user: NewUser) -> Result<(), CreateAccountError> {
        self.repo.save_user(user).await
    }
}
