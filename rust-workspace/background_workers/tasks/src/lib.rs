use crate::subscribe_to_location::{
    fetch_and_subscribe_to_locations, get_and_subscribe_to_nearby_location,
};

use crate::configuration::SETTINGS_CONFIG;
use crate::send_notifications::email::send_email_notification;
use anyhow::Context;
use celery::Celery;

use std::sync::Arc;
use text_search::search_locations_by_text;

pub mod configuration;
pub mod constants;
pub mod send_notifications;
pub mod subscribe_to_location;
pub mod text_search;
pub mod utils;

const QUEUE_NAME: &str = "celery";

pub async fn app() -> anyhow::Result<Arc<Celery>> {
    let redis_host = SETTINGS_CONFIG.redis.host.to_string();
    celery::app!(
        broker = RedisBroker { redis_host },
        tasks = [
            fetch_and_subscribe_to_locations,
            get_and_subscribe_to_nearby_location,
            search_locations_by_text,
            send_email_notification
        ],
        task_routes = [
            "*" => QUEUE_NAME,
            "fetch_and_subscribe_to_locations" => "fetch_and_subscribe_to_locations",
            "search_locations_by_text" => "search_locations_by_text",
            "send_email_notification" => "send_email_notification"
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
        acks_late = true
    )
    .await
    .context("Failed to initialize app")
}
