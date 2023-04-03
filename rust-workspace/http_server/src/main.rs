use std::env;
use std::sync::Arc;

use crate::use_case_app_container::UseCaseAppContainer;
use actix_web::{web, App, HttpServer};
use anyhow::Context;
use producer::producer::Producer;
use sqlx_postgres::repository::Repository;
use use_cases::AppImpl;

mod authentication;
mod errors;
mod routes;
mod use_case_app_container;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let repository = Repository::new().await?;
    let producer = Producer::new().await?;
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| 8080.to_string());
    let binding_address = format!("{host}:{port}");
    HttpServer::new(move || {
        let app = AppImpl::new(repository.clone(), producer.clone());
        let app_container = UseCaseAppContainer::new(app);
        App::new()
            .configure(routes::config)
            .app_data(web::Data::new(app_container))
    })
    .bind(binding_address)?
    .run()
    .await
    .context("Server failed to run")
}
