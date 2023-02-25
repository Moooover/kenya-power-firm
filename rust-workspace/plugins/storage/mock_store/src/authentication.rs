use crate::store::Repository;
use async_trait::async_trait;
use use_cases::authentication::{User, UserAuthenticationRepo};

#[async_trait]
impl UserAuthenticationRepo for Repository {
    async fn create_or_update_user(&self, user: User) -> anyhow::Result<()> {
        println!("User authentication details --- {user:?}");

        Ok(())
    }
}
