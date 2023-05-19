use celery::prelude::Task;
use celery::{prelude::TaskError, task::TaskResult};
use location_search::contracts::text_search::TextSearcher;

use crate::utils::callbacks::failure_callback;
use crate::utils::get_token::get_location_token;

#[celery::task(bind=true, max_retries = 200, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn search_locations_by_text(task: &Self, text: String) -> TaskResult<()> {
    let token_count = get_location_token().await?;

    if token_count < 0 {
        return Task::retry_with_countdown(task, 1);
    }

    let text_searcher = TextSearcher::new();
    // the client / Producer will query the response from the cache
    // no need to return the response here
    text_searcher
        .api_search(text)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    Ok(())
}
