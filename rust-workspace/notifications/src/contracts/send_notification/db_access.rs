use crate::contracts::send_notification::{AffectedSubscriber, LocationMatchedAndLineSchedule};
use crate::db_access::{DbAccess, SourceId};

use itertools::Itertools;
use shared_kernel::uuid_key;
use url::Url;
use uuid::Uuid;

pub struct SendNotificationsDbAccess {
    db_access: DbAccess,
}

impl AsRef<DbAccess> for SendNotificationsDbAccess {
    fn as_ref(&self) -> &DbAccess {
        &self.db_access
    }
}

uuid_key!(NotificationStrategyId);

pub trait Notification: std::fmt::Debug {
    fn subscriber(&self) -> AffectedSubscriber;

    fn locations_matched(&self) -> Vec<LocationMatchedAndLineSchedule>;

    fn url(&self) -> Url;
}

struct NotificationInsert {
    source: Uuid,
    directly_affected: bool,
    subscriber: Uuid,
    line: String,
    location_matched: Uuid,
    external_id: String,
    strategy_id: Uuid,
}

impl SendNotificationsDbAccess {
    pub fn new() -> Self {
        Self {
            db_access: DbAccess,
        }
    }

    pub async fn get_source_by_url(&self, url: &Url) -> anyhow::Result<SourceId> {
        self.db_access.get_source_by_url(url).await
    }

    pub async fn save_notification_sent(
        &self,
        notification: impl Notification,
        strategy: NotificationStrategyId,
        source: SourceId,
        external_id: String,
    ) -> anyhow::Result<()> {
        let is_directly_affected = matches!(
            notification.subscriber(),
            AffectedSubscriber::DirectlyAffected(_)
        );

        let subscriber = notification.subscriber().id().inner();
        let notification_inserts = notification
            .locations_matched()
            .iter()
            .map(|affected_line| NotificationInsert {
                source: source.inner(),
                directly_affected: is_directly_affected,
                subscriber,
                line: affected_line.line_schedule.line_name.clone(),
                location_matched: affected_line.location_id.inner(),
                external_id: external_id.clone(),
                strategy_id: strategy.inner(),
            })
            .collect_vec();

        let (
            source_ids,
            directly_affected,
            subscriber_id,
            line,
            strategy_id,
            location_id_matched,
            external_ids,
        ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = notification_inserts
            .iter()
            .map(|notification| {
                (
                    notification.source,
                    notification.directly_affected,
                    notification.subscriber,
                    notification.line.clone(),
                    notification.strategy_id,
                    notification.location_matched,
                    notification.external_id.clone(),
                )
            })
            .multiunzip();

        let pool = self.db_access.pool().await;
        sqlx::query!(
                "
                INSERT INTO communication.notifications(source_id, directly_affected, subscriber_id, line, strategy_id, location_id_matched, external_id)
                SELECT * FROM UNNEST($1::uuid[], $2::bool[], $3::uuid[], $4::text[], $5::uuid[], $6::uuid[], $7::text[]) ON CONFLICT DO NOTHING
                ",
                &source_ids[..],
                &directly_affected[..],
                &subscriber_id[..],
                &line[..],
                &strategy_id[..],
                &location_id_matched[..],
                &external_ids[..]
            )
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
