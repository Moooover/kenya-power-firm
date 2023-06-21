use crate::utils::callbacks::failure_callback;
use celery::error::TaskError;
use celery::prelude::Task;
use celery::task::TaskResult;

use crate::rate_limiting::EmailAPIRateLimiter;
use notifications::contracts::send_notification::AffectedSubscriberWithLocations;

#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn send_email_notification(
    task: &Self,
    data: AffectedSubscriberWithLocations,
) -> TaskResult<()> {
    let rate_limiter = EmailAPIRateLimiter::new().await;
    let rate_limit = rate_limiter.throttle().await?;
    if !rate_limit.action_is_allowed() {
        return Task::retry_with_countdown(task, rate_limit.retry_after() as u32);
    }

    let interactor =
        notifications::contracts::send_notification::email::EmailNotificationInteractor;

    interactor.send(data).await.map_err(|err| {
        TaskError::UnexpectedError(format!("Failed to send email notification: {}", err))
    })
}
