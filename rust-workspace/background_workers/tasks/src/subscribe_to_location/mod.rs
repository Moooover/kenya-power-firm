pub mod db;
pub mod nearby_locations;
mod primary_location;

use anyhow::Context;
use celery::export::async_trait;
use celery::prelude::*;
use entities::locations::ExternalLocationId;

use entities::subscriptions::SubscriberId;
use secrecy::ExposeSecret;
use shared_kernel::http_client::HttpClient;
use url::Url;

use crate::{
    send_notifications::email::send_email_notification, utils::get_token::get_location_token,
};

use serde::Deserialize;

use sqlx_postgres::cache::location_search::StatusCode;
use use_cases::subscriber_locations::subscribe_to_location::TaskId;

use crate::subscribe_to_location::nearby_locations::get_nearby_locations;
use crate::subscribe_to_location::primary_location::db_access::{
    LocationInput, LocationWithCoordinates,
};
use crate::utils::progress_tracking::{set_progress_status, TaskStatus};
use crate::{configuration::SETTINGS_CONFIG, utils::callbacks::failure_callback};

use self::{db::DB, nearby_locations::PrimaryLocation};

#[celery::task(max_retries = 200, bind = true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn fetch_and_subscribe_to_location(
    task: &Self,
    primary_location: ExternalLocationId,
    subscriber: SubscriberId,
    task_id: TaskId,
) -> TaskResult<()> {
    set_progress_status(
        task_id.as_ref(),
        TaskStatus::Pending.to_string(),
        |_| Ok(()),
    )
    .await
    .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
    let db = DB::new().await;
    let location_with_coordinates = db
        .find_location_id_and_coordinates(primary_location.clone())
        .await?;
    let location_with_coordinates = match location_with_coordinates {
        None => {
            let token_count = get_location_token().await?;

            if token_count < 0 {
                return Task::retry_with_countdown(task, 1);
            }

            get_location_from_api(primary_location).await?
        }
        Some(id) => id,
    };

    let _ = db
        .subscribe_to_primary_location(subscriber, location_with_coordinates.location_id)
        .await?;

    let direct_notification = db
        .subscriber_directly_affected(subscriber, location_with_coordinates.location_id)
        .await?;

    let subscriber_directly_affected = direct_notification.is_some();
    if let Some(notification) = direct_notification {
        let _ = task
            .request
            .app
            .send_task(send_email_notification::new(notification))
            .await
            .with_expected_err(|| "Failed to send task")?;
    }

    let primary_location = PrimaryLocation {
        location_id: location_with_coordinates.location_id.into(),
        latitude: location_with_coordinates.latitude,
        longitude: location_with_coordinates.longitude,
    };
    task.request
        .app
        .send_task(get_nearby_locations::new(
            primary_location,
            subscriber,
            subscriber_directly_affected,
            task_id.clone(),
        ))
        .await
        .with_expected_err(|| "Failed to send get_nearby_locations task")?;

    Ok(())
}

async fn save_location_returning_id_and_coordinates(
    location: LocationInput,
) -> TaskResult<LocationWithCoordinates> {
    let db = DB::new().await;
    let external_id = location.external_id.clone();
    let _ = db.insert_location(location).await?;
    let location_id = db.find_location_id_and_coordinates(external_id).await?;
    location_id
        .ok_or("Location not found")
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))
}

fn generate_url(id: ExternalLocationId) -> anyhow::Result<Url> {
    let place_details_path = "/place/details/json";

    let host = &SETTINGS_CONFIG.location.host;
    Url::parse_with_params(
        &format!("{}{}", host, place_details_path),
        &[
            ("key", SETTINGS_CONFIG.location.api_key.expose_secret()),
            ("place_id", &id.inner()),
        ],
    )
    .context("Failed to parse Url")
}

async fn get_location_from_api(
    external_id: ExternalLocationId,
) -> TaskResult<LocationWithCoordinates> {
    let url =
        generate_url(external_id).map_err(|err| TaskError::UnexpectedError(format!("{err}")))?;

    let location = get_place_details(url).await?;

    save_location_returning_id_and_coordinates(location.clone()).await
}

async fn get_place_details(url: Url) -> TaskResult<LocationInput> {
    #[derive(Deserialize, Debug, Clone)]
    struct ResponseResult {
        name: String,
        formatted_address: String,
        place_id: String,
    }
    #[derive(Deserialize, Debug, Clone)]
    struct Response {
        result: Option<ResponseResult>,
        status: StatusCode,
    }
    let result = HttpClient::get_json::<serde_json::Value>(url)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
    let response: Response = serde_json::from_value(result.clone())
        .with_expected_err(|| format!("Invalid response {result:?}"))?;

    if response.status.is_cacheable() {
        if let Some(response_result) = response.result {
            return Ok(LocationInput {
                external_id: ExternalLocationId::new(response_result.place_id),
                name: response_result.name,
                address: response_result.formatted_address,
                api_response: result,
            });
        }
    }
    Err(TaskError::UnexpectedError(format!(
        "Failed to get valid response {result:?}"
    )))
}
