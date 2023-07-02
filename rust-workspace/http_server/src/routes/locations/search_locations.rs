use actix_web::{web, HttpRequest};
use background_workers::producer::contracts::text_search::{LocationApiResponse, Status};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::app_container::Application;
use crate::{authentication::AuthenticatedUserInfo, errors::ApiError};

#[derive(Deserialize, Debug)]
struct Request {
    term: String,
}

impl From<Status> for StatusResponse {
    fn from(value: Status) -> Self {
        match value {
            Status::Pending => StatusResponse::Pending,
            Status::Success => StatusResponse::Success,
            Status::Failure => StatusResponse::Failure,
            Status::NotFound => StatusResponse::NotFound,
        }
    }
}

#[derive(Serialize)]
pub enum StatusResponse {
    Pending,
    Success,
    Failure,
    NotFound,
}

#[derive(Serialize)]
struct LocationSearchResponse {
    items: Vec<Location>,
    status: StatusResponse,
}

#[derive(Serialize)]
struct Location {
    name: String,
    id: String,
    address: String,
}

impl From<LocationApiResponse> for Location {
    fn from(value: LocationApiResponse) -> Self {
        Self {
            name: value.name,
            address: value.address,
            id: value.id.inner(),
        }
    }
}

#[tracing::instrument(err, skip(app), level = "info")]
async fn search_for_location(
    data: web::Query<Request>,
    app: web::Data<Application>,
    req: HttpRequest,
) -> Result<web::Json<LocationSearchResponse>, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let _ = app
        .subscribers
        .authenticate(user.external_id.as_ref())
        .await
        .map_err(ApiError::InternalServerError)?;
    let search_results = app
        .producer
        .search_for_location(&data.term)
        .await
        .map_err(ApiError::InternalServerError)?;
    Ok(web::Json(LocationSearchResponse {
        items: search_results
            .responses
            .into_iter()
            .map_into()
            .collect_vec(),
        status: search_results.status.into(),
    }))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/search").service(web::resource("").route(web::get().to(search_for_location))),
    );
}
