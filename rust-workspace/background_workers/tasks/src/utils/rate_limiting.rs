use crate::configuration::SETTINGS_CONFIG;
use crate::constants::{EMAIL_API_TOKEN_KEY, GOOGLE_API_TOKEN_KEY};
use anyhow::bail;
use anyhow::Context;
use celery::error::TaskError;
use celery::task::TaskResult;
use redis_client::client::{Client, CLIENT};

pub struct RateLimiter {
    client: Client,
}

impl RateLimiter {
    pub async fn new() -> Self {
        Self {
            client: CLIENT.get().await.clone(),
        }
    }

    pub async fn throttle(
        &self,
        key: &str,
        max_burst: i32,
        count_per_period: i32,
        period: i32,
        number_of_tokens: i32,
    ) -> anyhow::Result<RateLimitResponse> {
        /* CL.THROTTLE <key> <max_burst> <count per period> <period> [<quantity>] */
        /* Response is array of 5 integers. The meaning of each array item is:
         *  1. Whether the action was limited:
         *   - 0 indicates the action is allowed.
         *   - 1 indicates that the action was limited/blocked.
         *  2. The total limit of the key (max_burst + 1). This is equivalent to the common
         * X-RateLimit-Limit HTTP header.
         *  3. The remaining limit of the key. Equivalent to X-RateLimit-Remaining.
         *  4. The number of seconds until the user should retry, and always -1 if the action was allowed.
         * Equivalent to Retry-After.
         *  5. The number of seconds until the limit will reset to its maximum capacity. Equivalent to
         * X-RateLimit-Reset.
         */
        let response: Vec<Vec<i32>> = redis::pipe()
            .cmd("CL.THROTTLE")
            .arg(key)
            .arg(max_burst)
            .arg(count_per_period)
            .arg(period)
            .arg(number_of_tokens)
            .query_async(&mut self.client.connection())
            .await
            .context("Failed to CL.THROTTLE")?;

        let response = response.first().context("Empty ratelimiting response")?;
        if response.len() < 5 {
            bail!("Unexpected response length from CL.THROTTLE");
        }

        let action = match response[0] {
            0 => Action::Allowed,
            _ => Action::Limited,
        };

        Ok(RateLimitResponse {
            action,
            total_limit: response[1],
            remaining_limit: response[2],
            retry_after: response[3],
            reset_after: response[4],
        })
    }
}

#[derive(Copy, Clone)]
pub enum Action {
    Allowed,
    Limited,
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct RateLimitResponse {
    action: Action,
    total_limit: i32,
    remaining_limit: i32,
    retry_after: i32,
    reset_after: i32,
}

impl RateLimitResponse {
    pub(crate) fn action_is_allowed(&self) -> bool {
        matches!(self.action, Action::Allowed)
    }

    pub(crate) fn retry_after(&self) -> i32 {
        self.retry_after
    }
}

pub struct GoogleAPIRateLimiter {
    rate_limiter: RateLimiter,
}

impl GoogleAPIRateLimiter {
    pub async fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new().await,
        }
    }

    pub async fn throttle(&self, number_of_tokens: i32) -> TaskResult<RateLimitResponse> {
        self.rate_limiter
            .throttle(
                GOOGLE_API_TOKEN_KEY,
                2,
                SETTINGS_CONFIG.external_api_rate_limits.location as i32,
                1,
                number_of_tokens,
            )
            .await
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))
    }
}

pub struct EmailAPIRateLimiter {
    rate_limiter: RateLimiter,
}

impl EmailAPIRateLimiter {
    pub async fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new().await,
        }
    }

    pub async fn throttle(&self) -> TaskResult<RateLimitResponse> {
        self.rate_limiter
            .throttle(
                EMAIL_API_TOKEN_KEY,
                2,
                SETTINGS_CONFIG.external_api_rate_limits.email as i32,
                1,
                1,
            )
            .await
            .map_err(|err| TaskError::UnexpectedError(err.to_string()))
    }
}
