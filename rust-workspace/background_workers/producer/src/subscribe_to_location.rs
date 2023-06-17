use crate::producer::Producer;
use anyhow::Context;
use async_trait::async_trait;
use entities::subscriptions::SubscriberId;
use shared_kernel::location_ids::ExternalLocationId;
use std::str::FromStr;
use tasks::{
    subscribe_to_location::fetch_and_subscribe_to_location,
    utils::progress_tracking::{get_progress_status, TaskStatus},
};
use use_cases::search_for_locations::Status;
use use_cases::subscriber_locations::subscribe_to_location::{LocationSubscriber, TaskId};
use uuid::Uuid;

fn generate_task_id() -> TaskId {
    let key = Uuid::new_v4();
    key.to_string().into()
}

#[async_trait]
impl LocationSubscriber for Producer {
    async fn subscribe_to_location(
        &self,
        location: ExternalLocationId,
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<TaskId> {
        let task_id = generate_task_id();
        self.app
            .send_task(fetch_and_subscribe_to_location::new(
                location,
                subscriber_id,
                task_id.clone(),
            ))
            .await
            .context("Failed to send task")?;

        Ok(task_id)
    }

    async fn progress(&self, task_id: TaskId) -> anyhow::Result<Status> {
        let progress = get_progress_status::<String, _>(task_id.as_ref(), |val| {
            val.map(|value| {
                TaskStatus::from_str(&value)
                    .with_context(|| format!("Failed to convert to TaskStatus {value}"))
            })
            .transpose()
        })
        .await
        .map(|data| data.map(Into::into))?;

        Ok(progress.unwrap_or(Status::NotFound))
    }
}
