use std::sync::Arc;

use crate::use_case_app_container::UseCaseAppContainer;
use actix_web::{web, App, HttpServer};
use anyhow::Context;
use location_searcher::searcher::Searcher;
use sqlx_postgres::repository::Repository;
use use_cases::AppImpl;

mod authentication;
mod errors;
mod routes;
mod use_case_app_container;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let repository = Repository::new().await?;
    let location_searcher = Searcher::new(Arc::new(repository.clone()))?;

    HttpServer::new(move || {
        let app = AppImpl::new(repository.clone(), location_searcher.clone());
        let app_container = UseCaseAppContainer::new(app);
        App::new()
            .configure(routes::config)
            .app_data(web::Data::new(app_container))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
    .context("Server failed to run")
}
