use actix_web::{web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::app_container::Application;
use crate::{authentication::AuthenticatedUserInfo, errors::ApiError};

use super::search_locations::StatusResponse;

#[derive(Deserialize, Debug)]
struct LocationSubscriptionRequest {
    location: String,
}

#[derive(Serialize, Debug)]
struct SubscribeToLocationResponse {
    task_id: String,
}

#[tracing::instrument(err, skip(app), level = "info")]
async fn subscribe_to_location(
    data: web::Json<LocationSubscriptionRequest>,
    app: web::Data<Application>,
    req: HttpRequest,
) -> Result<web::Json<SubscribeToLocationResponse>, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let subscriber = app
        .subscribers
        .authenticate(user.external_id.as_ref())
        .await
        .map_err(ApiError::InternalServerError)?;
    let task_id = app
        .producer
        .subscribe_to_location(data.into_inner().location.as_ref(), subscriber)
        .await
        .map_err(ApiError::InternalServerError)?;

    Ok(web::Json(SubscribeToLocationResponse {
        task_id: task_id.to_string(),
    }))
}

#[derive(Serialize)]
struct StatusWrapper {
    data: StatusResponse,
}

#[tracing::instrument(err, skip(app), level = "info")]
async fn get_progress_status(
    app: web::Data<Application>,
    task_id: web::Path<String>,
    req: HttpRequest,
) -> Result<web::Json<StatusWrapper>, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let _ = app
        .subscribers
        .authenticate(user.external_id.as_ref())
        .await
        .map_err(ApiError::InternalServerError)?;
    let result = app
        .producer
        .location_subscription_progress(task_id.into_inner())
        .await
        .map_err(ApiError::InternalServerError)?;
    Ok(web::Json(StatusWrapper {
        data: result.into(),
    }))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subscribe")
            .service(web::resource("").route(web::post().to(subscribe_to_location)))
            .service(
                web::resource("/progress/{task_id}").route(web::get().to(get_progress_status)),
            ),
    );
}
