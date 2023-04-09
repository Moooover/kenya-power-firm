use crate::constants::{EMAIL_API_TOKEN_KEY, GOOGLE_API_TOKEN_KEY};
use celery::error::TaskError;
use celery::prelude::TaskResult;
use redis_client::client::CLIENT;

pub async fn get_location_token() -> TaskResult<i32> {
    get_token(GOOGLE_API_TOKEN_KEY).await
}

pub async fn get_email_token() -> TaskResult<i32> {
    get_token(EMAIL_API_TOKEN_KEY).await
}

async fn get_token(key: &str) -> TaskResult<i32> {
    let token_client = CLIENT.get().await;

    token_client
        .get_token_count(key)
        .await
        .map_err(|err| TaskError::ExpectedError(format!("{err}")))
}
