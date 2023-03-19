use crate::constants::GOOGLE_API_TOKEN_KEY;
use celery::error::TaskError;
use celery::prelude::TaskResult;
use redis_client::client::CLIENT;

pub async fn get_token_count() -> TaskResult<i32> {
    let token_client = CLIENT.get().await;

    token_client
        .get_token(GOOGLE_API_TOKEN_KEY)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))
}
