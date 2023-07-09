use crate::producer::contracts::text_search::Status;
use crate::producer::Producer;
use crate::tasks::TaskId;
use crate::{
    tasks::subscribe_to_location::fetch_and_subscribe_to_location,
    utils::progress_tracking::{get_progress_status, TaskStatus},
};
use anyhow::Context;

use shared_kernel::location_ids::ExternalLocationId;
use shared_kernel::subscriber_id::SubscriberId;
use std::str::FromStr;
use uuid::Uuid;

fn generate_task_id() -> TaskId {
    let key = Uuid::new_v4();
    key.to_string().into()
}

impl Producer {
    pub async fn subscribe_to_location(
        &self,
        location: impl Into<ExternalLocationId>,
        subscriber_id: SubscriberId,
    ) -> anyhow::Result<TaskId> {
        let location = location.into();
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

    pub async fn location_subscription_progress(
        &self,
        task_id: impl Into<TaskId>,
    ) -> anyhow::Result<Status> {
        let task_id = task_id.into();
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
