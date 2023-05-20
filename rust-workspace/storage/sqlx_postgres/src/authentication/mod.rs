use crate::repository::Repository;
use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::details::SubscriberDetails;
use use_cases::authentication::SubscriberAuthenticationRepo;

#[async_trait]
impl SubscriberAuthenticationRepo for Repository {
    #[tracing::instrument(err, skip(self), level = "info")]
    async fn create_or_update_subscriber(
        &self,
        subscriber: SubscriberDetails,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
        INSERT INTO public.subscriber (name, email, external_id) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (external_id) 
        DO UPDATE SET name = EXCLUDED.name, email = EXCLUDED.email, last_login = now();
        "#,
            subscriber.name.as_ref(),
            subscriber.email.as_ref(),
            subscriber.external_id.as_ref()
        )
        .execute(self.pool())
        .await
        .context("Failed to create or insert subscriber")
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::Repository;
    use entities::subscriptions::details::{
        SubscriberDetails, SubscriberEmail, SubscriberExternalId, SubscriberName,
    };

    use use_cases::authentication::SubscriberAuthenticationRepo;

    #[tokio::test]
    async fn test_that_create_subscriber_works() {
        let subscriber_details = SubscriberDetails {
            name: SubscriberName::try_from("test_user".to_string()).unwrap(),
            email: SubscriberEmail::try_from("test_user@gmail.com".to_string()).unwrap(),
            external_id: SubscriberExternalId::try_from("external|id".to_string()).unwrap(),
        };
        let repo = Repository::new_test_repo().await;

        let result = repo.create_or_update_subscriber(subscriber_details).await;

        assert!(result.is_ok())
    }
}
