use anyhow::Context;
use redis::{FromRedisValue, ToRedisArgs};
use redis_client::client::CLIENT;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use use_cases::search_for_locations::Status;

const PROGRESS_TRACKING_PREFIX: &str = "progress";

#[derive(Debug, PartialEq, EnumString, Display, Copy, Clone)]
pub enum TaskStatus {
    Pending,
    Success,
    Failure,
    NotFound,
}

impl From<Status> for TaskStatus {
    fn from(value: Status) -> Self {
        match value {
            Status::Pending => TaskStatus::Pending,
            Status::Success => TaskStatus::Success,
            Status::Failure => TaskStatus::Failure,
            Status::NotFound => TaskStatus::NotFound,
        }
    }
}

impl From<TaskStatus> for Status {
    fn from(value: TaskStatus) -> Self {
        match value {
            TaskStatus::Pending => Status::Pending,
            TaskStatus::Success => Status::Success,
            TaskStatus::Failure => Status::Failure,
            TaskStatus::NotFound => Self::NotFound,
        }
    }
}

pub fn generate_key(key: &str) -> String {
    format!("{PROGRESS_TRACKING_PREFIX}:{key}")
}

pub async fn set_progress_status<S, F, C>(key: &str, status: S, mapper: F) -> anyhow::Result<C>
where
    F: FnOnce(S) -> anyhow::Result<C>,
    S: FromRedisValue + ToRedisArgs,
{
    let progress_tracker = CLIENT.get().await;
    let key = generate_key(key);
    progress_tracker
        .set_status::<_, S>(key, status)
        .await
        .map(mapper)?
}

pub async fn get_progress_status<V, F>(key: &str, mapper: F) -> anyhow::Result<Option<TaskStatus>>
where
    F: FnOnce(Option<V>) -> anyhow::Result<Option<TaskStatus>>,
    V: FromRedisValue,
{
    let progress_tracker = CLIENT.get().await;
    let key = generate_key(key);

    let progress = progress_tracker.get_status::<_, V>(key).await?;
    mapper(progress)
}
