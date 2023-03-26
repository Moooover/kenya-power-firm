use anyhow::{Context, Error};
use lazy_static::lazy_static;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde::Serialize;
use shared_kernel::configuration::config;
use shared_kernel::http_client::HttpClient;
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize)]
struct Settings {
    email: EmailConfig,
}
#[derive(Deserialize)]
struct EmailConfig {
    host: Url,
    auth_token: Secret<String>,
    address_to_alert: String,
    dry_run_template_id: String,
}

lazy_static! {
    static ref EMAIL: EmailConfig = config::<Settings>()
        .expect("Expected to have the configuration set")
        .email;
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub error_message: String,
}

#[derive(Serialize, Deserialize)]
struct To {
    pub email: String,
}
#[derive(Serialize, Deserialize)]
struct Message {
    pub to: To,
    pub template: String,
    pub data: Data,
}

#[derive(Serialize, Deserialize)]
struct Root {
    pub message: Message,
}

pub async fn send_alert(alert_message: impl ToString) -> anyhow::Result<()> {
    let body = Root {
        message: Message {
            to: To {
                email: EMAIL.address_to_alert.to_string(),
            },
            template: EMAIL.dry_run_template_id.to_string(),
            data: Data {
                error_message: alert_message.to_string(),
            },
        },
    };
    let as_str = serde_json::to_string(&body).context("Failed to convert to string")?;

    let body = serde_json::from_str(&as_str).context("Failed to convert from str")?;

    let auth_token = EMAIL.auth_token.expose_secret();
    let bearer_token = format!("Bearer {auth_token}");

    let headers = HashMap::from([("Authorization", bearer_token)]);
    HttpClient::post_json::<serde_json::Value>(EMAIL.host.clone(), headers, body)
        .await
        .map(|_| ())
        .context("Failed to send alert")
}
