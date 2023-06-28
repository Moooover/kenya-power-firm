use crate::contracts::SubscriberContracts;
use crate::find_subscriber::{
    SubscriberDetails, SubscriberEmail, SubscriberExternalId, SubscriberName,
};

#[derive(Debug)]
pub struct SubscriberInput {
    pub name: String,
    pub email: String,
    pub external_id: String,
}

impl SubscriberContracts {
    #[tracing::instrument(err, level = "info")]
    pub async fn create_or_update_subscriber(input: SubscriberInput) -> anyhow::Result<()> {
        let external_id = SubscriberExternalId::try_from(input.external_id)
            .map_err(|err| anyhow::anyhow!(err))?;
        let name = SubscriberName::try_from(input.name).map_err(|err| anyhow::anyhow!(err))?;
        let email = SubscriberEmail::try_from(input.email).map_err(|err| anyhow::anyhow!(err))?;

        let details = SubscriberDetails {
            name,
            email,
            external_id,
        };

        create_or_update_subscriber::execute(details).await
    }
}

mod create_or_update_subscriber {
    use crate::db_access::DbAccess;
    use crate::find_subscriber::SubscriberDetails;
    use anyhow::Context;

    #[tracing::instrument(err, level = "info")]
    pub async fn execute(subscriber: SubscriberDetails) -> anyhow::Result<()> {
        let db = DbAccess {};
        let pool = db.pool().await;
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
        .execute(pool.as_ref())
        .await
        .context("Failed to create or insert subscriber")
        .map(|_| ())
    }
}
