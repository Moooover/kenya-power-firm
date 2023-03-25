use crate::{
    configuration::{REPO, SETTINGS_CONFIG},
    utils::callbacks::failure_callback,
};
use celery::error::TaskError;
use celery::prelude::TaskResultExt;
use celery::task::TaskResult;

use entities::{
    locations::LocationName,
    notifications::Notification,
    power_interruptions::location::{AffectedLine, NairobiTZDateTime},
    subscriptions::AffectedSubscriber,
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::http_client::HttpClient;
use std::collections::{HashMap, HashSet};
use url::Url;

const TEMPLATE: &str = "565V4THQRHM19SMJNC6WFKZRVWGR";

#[derive(Serialize, Deserialize)]
enum AffectedState {
    #[serde(rename = "directly affected")]
    DirectlyAffected,
    #[serde(rename = "potentially affected")]
    PotentiallyAffected,
}

#[derive(Serialize, Deserialize)]
struct AffectedLocation {
    pub location: String,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
}

impl AffectedLocation {
    fn generate(data: &AffectedLine<NairobiTZDateTime>, location: &LocationName) -> Self {
        let date_in_nairobi_time = data.time_frame.from.to_date_time();
        let date = date_in_nairobi_time.format("%d/%m/%Y");
        let start_time = date_in_nairobi_time.format("%H:%M");
        let end_time = data.time_frame.to.to_date_time().format("%H:%M");
        Self {
            location: location.to_string(),
            date: date.to_string(),
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct MessageData {
    pub recipient_name: String,
    pub affected_state: AffectedState,
    pub link: String,
    pub affected_locations: Vec<AffectedLocation>,
}

#[derive(Serialize, Deserialize)]
struct To {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    pub to: To,
    pub template: String,
    pub data: MessageData,
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub message: Message,
}

impl Data {
    fn as_json(&self) -> serde_json::Result<serde_json::Value> {
        let as_str = serde_json::to_string(self)?;
        serde_json::from_str(&as_str)
    }
}

#[derive(Deserialize)]
struct Response {
    #[serde(rename = "requestId")]
    request_id: String,
}

#[celery::task(max_retries = 200, bind=true, retry_for_unexpected = false, on_failure = failure_callback)]
pub async fn send_email_notification(task: &Self, notification: Notification) -> TaskResult<()> {
    let repo = REPO.get().await;
    let notification = repo
        .filter_email_notification_by_those_already_sent(notification)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    if notification.already_sent() {
        return Ok(());
    }

    let body = generate_email_body(&notification).await?;
    let settings = &SETTINGS_CONFIG.email;
    let url = Url::parse(&settings.host)
        .with_unexpected_err(|| format!("Invalid url {}", &settings.host))?;

    let auth_token = settings.auth_token.expose_secret();
    let bearer_token = format!("Bearer {auth_token}");

    let headers = HashMap::from([("Authorization", bearer_token)]);

    let body = body
        .as_json()
        .with_unexpected_err(|| "Failed to convert the body to a valid json")?;

    let response = HttpClient::post_json::<Response>(url, headers, body)
        .await
        .map_err(|err| TaskError::ExpectedError(err.to_string()))?;

    repo.save_email_notification_sent(&notification, response.request_id)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))
}

async fn generate_email_body(notification: &Notification) -> TaskResult<Data> {
    let repo = REPO.get().await;
    let subscriber_id = notification.subscriber.id();
    let subscriber = repo
        .find_subscriber_by_id(subscriber_id)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;
    let locations = notification
        .lines
        .iter()
        .map(|line| line.location_matched)
        .collect::<HashSet<_>>();

    let locations = repo
        .get_locations_by_ids(locations)
        .await
        .map_err(|err| TaskError::UnexpectedError(err.to_string()))?;

    let affected_locations = notification
        .lines
        .iter()
        .filter_map(|affected_line| {
            locations
                .get(&affected_line.location_matched)
                .map(|location_details| {
                    AffectedLocation::generate(affected_line, &location_details.name)
                })
        })
        .collect::<Vec<_>>();

    if affected_locations.is_empty() {
        return Err(TaskError::UnexpectedError(format!(
            "The affected locations cannot be empty {notification:?}"
        )));
    }

    let message = Data {
        message: Message {
            to: To {
                email: subscriber.email.to_string(),
            },
            template: TEMPLATE.to_string(),
            data: MessageData {
                recipient_name: subscriber.name.to_string(),
                affected_state: notification.subscriber.into(),
                link: notification.url.to_string(),
                affected_locations,
            },
        },
    };

    Ok(message)
}

impl From<AffectedSubscriber> for AffectedState {
    fn from(value: AffectedSubscriber) -> Self {
        match value {
            AffectedSubscriber::DirectlyAffected(_) => AffectedState::DirectlyAffected,
            AffectedSubscriber::PotentiallyAffected(_) => AffectedState::PotentiallyAffected,
        }
    }
}
