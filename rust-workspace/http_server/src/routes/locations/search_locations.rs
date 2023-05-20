use actix_web::{web, HttpRequest};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use use_cases::search_for_locations::{LocationApiResponse, Status};

use crate::{
    authentication::AuthenticatedUserInfo, errors::ApiError,
    use_case_app_container::UseCaseAppContainer,
};

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
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<web::Json<LocationSearchResponse>, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let interactor = app.get_client().location_searcher();
    let results = interactor
        .search(&user, data.into_inner().term)
        .await
        .map_err(ApiError::InternalServerError)?;
    let items = results.responses.into_iter().map_into().collect();
    Ok(web::Json(LocationSearchResponse {
        items,
        status: results.status.into(),
    }))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/search").service(web::resource("").route(web::get().to(search_for_location))),
    );
}
