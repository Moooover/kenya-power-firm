use crate::db_access::DbAccess;
use anyhow::Context;
use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;

pub struct UnsubscribeDbAccess {
    db: DbAccess,
}

impl UnsubscribeDbAccess {
    pub fn new() -> Self {
        Self { db: DbAccess }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn unsubscribe(
        &self,
        subscriber_id: SubscriberId,
        location_id: LocationId,
    ) -> anyhow::Result<()> {
        let subscriber_id = subscriber_id.inner();
        let location_id = location_id.inner();

        let pool = self.db.pool().await;

        let _ = sqlx::query!(
            "DELETE FROM location.subscriber_locations 
            WHERE subscriber_id = $1 AND location_id = $2",
            subscriber_id,
            location_id
        )
        .execute(pool.as_ref())
        .await
        .context("Failed to unsubscribe to location")?;

        Ok(())
    }
}
