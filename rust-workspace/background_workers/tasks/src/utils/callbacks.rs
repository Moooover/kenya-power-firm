use celery::{prelude::TaskError, task::Task};

pub async fn failure_callback<T: Task>(task: &T, err: &TaskError) {
    match err {
        TaskError::TimeoutError => println!("Oops! Task {} timed out!", task.name()),
        _ => println!("Hmm task {} failed with {:?}", task.name(), err),
    };
}
