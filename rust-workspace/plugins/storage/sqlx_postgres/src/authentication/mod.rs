use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use use_cases::authentication::{User, UserAuthenticationRepo};

#[async_trait]
impl UserAuthenticationRepo for Repository {
    async fn create_or_update_user(&self, user: User) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
        INSERT INTO public.subscriber (name, email, external_id) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (external_id) 
        DO UPDATE SET name = EXCLUDED.name, email = EXCLUDED.email, updated_at = now();
        "#,
            user.details.name,
            user.details.email,
            user.external_id.to_string()
        )
        .execute(self.pool())
        .await
        .context("Failed to create or insert user")
        .map(|_| ())
    }
}
