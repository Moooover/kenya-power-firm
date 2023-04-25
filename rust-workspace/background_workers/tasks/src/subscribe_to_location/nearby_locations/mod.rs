mod db_access;

use crate::configuration::SETTINGS_CONFIG;
use crate::subscribe_to_location::db::DB;
use crate::subscribe_to_location::nearby_locations::db_access::NearbyLocationId;
use crate::utils::callbacks::failure_callback;
use celery::prelude::*;
use entities::locations::LocationId;
use entities::subscriptions::SubscriberId;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use shared_kernel::http_client::HttpClient;
use sqlx_postgres::cache::location_search::StatusCode;
use url::Url;
use use_cases::subscriber_locations::subscribe_to_location::TaskId;

use crate::send_notifications::email::send_email_notification;
use crate::utils::get_token::get_location_token;
use crate::utils::progress_tracking::{set_progress_status, TaskStatus};

#[derive(Deserialize, Serialize, Clone)]
pub struct PrimaryLocation {
    pub location_id: LocationId,
    pub latitude: f64,
    pub longitude: f64,
}

#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn get_nearby_locations(
    task: &Self,
    primary_location: PrimaryLocation,
    subscriber_id: SubscriberId,
    subscriber_directly_affected: bool,
    task_id: TaskId,
) -> TaskResult<()> {
    let url = generate_url(&primary_location)?;
    let db = DB::new().await;
    let already_fetched = db.is_nearby_locations_already_fetched(url.clone()).await?;

    let nearby_location_id = match already_fetched {
        None => {
            let token = get_location_token().await?;
            if token < 0 {
                return Task::retry_with_countdown(task, 1);
            }
            get_and_save_nearby_locations(&db, url, primary_location.location_id).await?
        }
        Some(id) => id,
    };

    if subscriber_directly_affected {
        set_progress_status(
            task_id.as_ref(),
            TaskStatus::Success.to_string(),
            |_| Ok(()),
        )
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
        return Ok(());
    }

    let notification = db
        .is_potentially_affected(subscriber_id, nearby_location_id)
        .await?;

    if let Some(notification) = notification {
        let _ = task
            .request
            .app
            .send_task(send_email_notification::new(notification))
            .await
            .with_expected_err(|| "Failed to send task")?;
    }

    set_progress_status(
        task_id.as_ref(),
        TaskStatus::Success.to_string(),
        |_| Ok(()),
    )
    .await
    .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
    Ok(())
}

fn generate_url(primary_location: &PrimaryLocation) -> TaskResult<Url> {
    let nearby_locations_path = "/place/nearbysearch/json?rankby=distance";
    let host = &SETTINGS_CONFIG.location.host;
    Url::parse_with_params(
        &format!("{}{}", host, nearby_locations_path),
        &[
            (
                "location",
                &format!(
                    "{} {}",
                    primary_location.latitude, primary_location.longitude
                ),
            ),
            ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
        ],
    )
    .with_unexpected_err(|| "Failed to parse nearby_location URL")
}

async fn get_nearby_locations_from_api(url: Url) -> TaskResult<serde_json::Value> {
    let raw_response = HttpClient::get_json::<serde_json::Value>(url)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    #[derive(Deserialize, Debug, Clone)]
    struct Response {
        status: StatusCode,
    }

    let response: Response = serde_json::from_value(raw_response.clone())
        .with_expected_err(|| format!("Invalid response {raw_response:?}"))?;

    if response.status.is_cacheable() {
        return Ok(raw_response);
    }
    Err(TaskError::UnexpectedError(format!(
        "Failed to get valid response {raw_response:?}"
    )))
}

async fn get_and_save_nearby_locations(
    db: &DB,
    url: Url,
    primary_location: LocationId,
) -> TaskResult<NearbyLocationId> {
    let api_response = get_nearby_locations_from_api(url.clone()).await?;

    db.save_nearby_locations(url, primary_location, api_response)
        .await
}
