use async_trait::async_trait;

pub struct User {
    external_id: String,
}

#[async_trait]
pub trait LoginInteractor {
    async fn login(&self, user: User) -> anyhow::Result<()>;
}
