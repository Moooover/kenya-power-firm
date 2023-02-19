use async_trait::async_trait;
use std::sync::Arc;

pub struct User {
    pub external_id: String,
}

#[async_trait]
pub trait LoginInteractor {
    async fn login(&self, user: User) -> Result<(), LoginError>;
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("account not found")]
    AccountNotFound,
    #[error("internal server error")]
    InternalServerError(#[from] anyhow::Error),
}

#[async_trait]
pub trait UserLoginRepo {
    async fn login(&self, user: User) -> Result<(), LoginError>;
}

pub struct LoginInteractorImpl {
    repo: Arc<dyn UserLoginRepo>,
}

#[async_trait]
impl LoginInteractor for LoginInteractorImpl {
    async fn login(&self, user: User) -> Result<(), LoginError> {
        self.repo.login(user).await
    }
}
