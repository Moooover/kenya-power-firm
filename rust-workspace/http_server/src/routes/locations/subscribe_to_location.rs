use actix_web::{web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::{
    authentication::AuthenticatedUserInfo, errors::ApiError,
    use_case_app_container::UseCaseAppContainer,
};

use super::search_locations::StatusResponse;

#[derive(Deserialize)]
struct LocationSubscriptionRequest {
    location: String,
}

#[derive(Serialize)]
struct SubscribeToLocationResponse {
    task_id: String,
}

async fn subscribe_to_location(
    data: web::Json<LocationSubscriptionRequest>,
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<web::Json<SubscribeToLocationResponse>, ApiError> {
    let interactor = app.get_client().subscribe_to_location();
    let user: AuthenticatedUserInfo = (&req).try_into()?;

    let data = data.into_inner();
    let id = interactor
        .subscribe(&user, data.location)
        .await
        .map_err(ApiError::InternalServerError)?;

    Ok(web::Json(SubscribeToLocationResponse {
        task_id: id.to_string(),
    }))
}

#[derive(Serialize)]
struct StatusWrapper {
    data: StatusResponse,
}

async fn get_progress_status(
    app: web::Data<UseCaseAppContainer>,
    task_id: web::Path<String>,
    req: HttpRequest,
) -> Result<web::Json<StatusWrapper>, ApiError> {
    let interactor = app.get_client().subscribe_to_location();
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let task_id = task_id.into_inner().into();
    let result = interactor
        .progress(&user, task_id)
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
