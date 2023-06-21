use celery::{prelude::TaskError, task::Task};
use tracing::error;

pub async fn failure_callback<T: Task>(task: &T, err: &TaskError) {
    match err {
        TaskError::TimeoutError => error!("Oops! Task {} timed out!", task.name()),
        _ => error!("Hmm task {} failed with {:?}", task.name(), err),
    };
}
