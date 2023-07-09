use crate::find_subscriber::{
    SubscriberDetails, SubscriberEmail, SubscriberExternalId, SubscriberName,
};
use crate::{contracts::SubscribersSubsystem, db_access::DbAccess};
use anyhow::Context;

#[derive(Debug)]
pub struct SubscriberInput {
    pub name: String,
    pub email: String,
    pub external_id: String,
}

impl SubscribersSubsystem {
    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn create_or_update_subscriber(&self, input: SubscriberInput) -> anyhow::Result<()> {
        let external_id = SubscriberExternalId::try_from(input.external_id)
            .map_err(|err| anyhow::anyhow!(err))?;
        let name = SubscriberName::try_from(input.name).map_err(|err| anyhow::anyhow!(err))?;
        let email = SubscriberEmail::try_from(input.email).map_err(|err| anyhow::anyhow!(err))?;

        let details = SubscriberDetails {
            name,
            email,
            external_id,
        };

        let db = DbAccess {};
        let pool = db.pool().await;
        sqlx::query!(
            r#"
        INSERT INTO public.subscriber (name, email, external_id) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (external_id) 
        DO UPDATE SET name = EXCLUDED.name, email = EXCLUDED.email, last_login = now();
        "#,
            details.name.as_ref(),
            details.email.as_ref(),
            details.external_id.as_ref()
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to create or insert subscriber")
        .map(|_| ())
    }
}
