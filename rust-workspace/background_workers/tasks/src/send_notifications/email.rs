use crate::{
    utils::callbacks::failure_callback,
};
use celery::error::TaskError;
use celery::prelude::{Task};
use celery::task::TaskResult;

use crate::utils::get_token::get_email_token;



use notifications::contracts::send_notification::AffectedSubscriberWithLocations;









#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn send_email_notification(
    task: &Self,
    data: AffectedSubscriberWithLocations,
) -> TaskResult<()> {
    let token_count = get_email_token().await?;
    if token_count < 0 {
        return Task::retry_with_countdown(task, 1);
    }
    let interactor =
        notifications::contracts::send_notification::email::EmailNotificationInteractor;

    interactor.send(data).await.map_err(|err| {
        TaskError::UnexpectedError(format!("Failed to send email notification: {}", err))
    })
}
