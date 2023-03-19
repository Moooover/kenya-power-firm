use crate::location_details::{
    fetch_and_subscribe_to_locations, get_and_subscribe_to_nearby_location,
};

use anyhow::Context;
use celery::Celery;
use std::sync::Arc;
use text_search::search_locations_by_text;

pub mod configuration;
pub mod constants;
pub mod location_details;
pub mod text_search;
pub mod utils;

const QUEUE_NAME: &str = "celery";

pub async fn app() -> anyhow::Result<Arc<Celery>> {
    celery::app!( // TODO: Fix address, pass it via configuration.
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            fetch_and_subscribe_to_locations,
            get_and_subscribe_to_nearby_location,
            search_locations_by_text
        ],
        task_routes = [
            "*" => QUEUE_NAME,
            "fetch_and_subscribe_to_locations" => "fetch_and_subscribe_to_locations",
            "search_locations_by_text" => "search_locations_by_text"
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
        acks_late = true
    ).await.context("Failed to initialize app")
}
