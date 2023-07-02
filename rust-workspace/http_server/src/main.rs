use actix_cors::Cors;
use std::env;

use crate::app_container::Application;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{http, web, App, HttpServer};

use anyhow::{Context, Error};
use background_workers::producer::Producer;
use location_subscription::contracts::LocationSubscriptionSubSystem;
use sqlx_postgres::migrations::MigrationManager;
use tracing::info;

mod app_container;
mod authentication;
mod errors;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_kernel::tracing::config_telemetry();
    start().await?;
    shared_kernel::tracing::shutdown_global_tracer_provider();
    Ok(())
}

async fn start() -> Result<(), Error> {
    {
        let migration_manager = MigrationManager::new().await?;
        migration_manager.migrate().await?;
    }

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| 8080.to_string());
    let binding_address = format!("{host}:{port}");
    info!("Starting server on {}", binding_address);
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .finish()
        .context("Failed to build governor config")?;
    let producer = Producer::new().await?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://localhost:4173")
            .allowed_origin("https://kplc-alerts.onrender.com")
            .allowed_origin("https://kplc-alerts-7pbm.onrender.com")
            .allow_any_method()
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ]);

        let application = Application::new(producer.clone());
        App::new()
            .wrap(cors)
            .wrap(actix_web_opentelemetry::RequestTracing::new())
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(Governor::new(&governor_conf))
            .configure(routes::config)
            .app_data(web::Data::new(application))
    })
    .bind(binding_address)?
    .run()
    .await
    .context("Server failed to run")
}
