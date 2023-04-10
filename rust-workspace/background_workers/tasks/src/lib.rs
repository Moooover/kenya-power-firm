use crate::subscribe_to_location::{
    fetch_and_subscribe_to_locations, get_and_subscribe_to_nearby_location,
};

use crate::configuration::SETTINGS_CONFIG;
use crate::send_notifications::email::send_email_notification;
use anyhow::Context;
use celery::Celery;

use std::sync::Arc;
use text_search::search_locations_by_text;
extern crate num_cpus;

pub mod configuration;
pub mod constants;
pub mod send_notifications;
pub mod subscribe_to_location;
pub mod text_search;
pub mod utils;

const QUEUE_NAME: &str = "celery";

pub async fn app() -> anyhow::Result<Arc<Celery>> {
    let redis_host = SETTINGS_CONFIG.redis.host.to_string();
    let pre_fetch_count = get_pre_fetch_count();

    celery::app!(
        broker = RedisBroker { redis_host },
        tasks = [
            fetch_and_subscribe_to_locations,
            get_and_subscribe_to_nearby_location,
            search_locations_by_text,
            send_email_notification
        ],
        task_routes = [
            "fetch_and_subscribe_to_locations" => "locations_queue",
            "search_locations_by_text" => "locations_queue",
            "send_email_notification" => "email_notifications_queue",
            "*" => QUEUE_NAME
        ],
        prefetch_count = pre_fetch_count,
        heartbeat = Some(10),
        acks_late = true
    )
    .await
    .context("Failed to initialize app")
}

fn get_pre_fetch_count() -> u16 {
    // https://rusty-celery.github.io/best-practices/index.html#prefetch-count
    //A good starting point for prefetch_count would be either 100 x NUM_CPUS for IO-bound tasks
    // or 2 * NUM_CPUS for CPU-bound tasks.
    let cpus = num_cpus::get();

    (cpus * 100) as u16
}
