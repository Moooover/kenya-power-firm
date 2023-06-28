use crate::db_access::DbAccess;
use anyhow::{anyhow, Context};
use shared_kernel::non_empty_string;
use shared_kernel::subscriber_id::SubscriberId;

pub struct FindSubscriber {
    db: DbAccess,
}

impl FindSubscriber {
    pub fn new() -> Self {
        Self { db: DbAccess {} }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn find_by_external_id(
        &self,
        external_id: SubscriberExternalId,
    ) -> anyhow::Result<SubscriberId> {
        let pool = self.db.pool().await;
        let result = sqlx::query!(
            "
            SELECT id FROM public.subscriber WHERE external_id = $1
            ",
            external_id.as_ref()
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch subscriber details")?;

        Ok(result.id.into())
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn find_subscriber_by_id(
        &self,
        id: SubscriberId,
    ) -> anyhow::Result<SubscriberDetails> {
        let id = id.inner();
        let pool = self.db.pool().await;
        let result = sqlx::query!(
            "
            SELECT * FROM public.subscriber WHERE id = $1
            ",
            id
        )
        .fetch_one(pool.as_ref())
        .await
        .context("Failed to fetch subscriber details")?;
        let name = SubscriberName::try_from(result.name).map_err(|err| anyhow::anyhow!(err))?;
        let email = SubscriberEmail::try_from(result.email).map_err(|err| anyhow!(err))?;
        let external_id =
            SubscriberExternalId::try_from(result.external_id).map_err(|err| anyhow!(err))?;
        let subscriber = SubscriberDetails {
            name,
            email,
            external_id,
        };

        Ok(subscriber)
    }
}

non_empty_string!(SubscriberName);
non_empty_string!(SubscriberEmailInner);
non_empty_string!(SubscriberExternalId);

#[derive(Debug)]
pub struct SubscriberEmail(SubscriberEmailInner);

impl ToString for SubscriberEmail {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct SubscriberDetails {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
    pub external_id: SubscriberExternalId,
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use validator::validate_email;
        let non_empty_string = SubscriberEmailInner::try_from(value)?;

        let is_valid = validate_email(non_empty_string.as_ref());
        if is_valid {
            return Ok(SubscriberEmail(non_empty_string));
        }
        Err(format!("{} is an invalid email", non_empty_string.as_ref()))
    }
}
