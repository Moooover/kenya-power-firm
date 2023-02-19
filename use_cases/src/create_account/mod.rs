use async_trait::async_trait;

pub struct NewUser {
    pub name: String,
    pub email: String,
    pub external_id: String,
}

#[async_trait]
pub trait CreateAccountInteractor {
    async fn create_account(&self, user: NewUser) -> anyhow::Result<()>;
}
