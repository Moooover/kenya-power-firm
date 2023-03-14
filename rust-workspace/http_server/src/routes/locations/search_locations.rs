use actix_web::{web, HttpRequest};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use use_cases::search_for_locations::LocationApiResponse;

use crate::{
    authentication::AuthenticatedUserInfo, errors::ApiError,
    use_case_app_container::UseCaseAppContainer,
};

#[derive(Deserialize)]
struct Request {
    term: String,
}

#[derive(Serialize)]

struct LocationSearchResponse {
    items: Vec<Location>,
}

#[derive(Serialize)]
struct Location {
    name: String,
    id: String,
}

impl From<LocationApiResponse> for Location {
    fn from(value: LocationApiResponse) -> Self {
        Self {
            name: value.name,
            id: value.id.inner(),
        }
    }
}

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
    let items = results.into_iter().map_into().collect();
    Ok(web::Json(LocationSearchResponse { items }))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/search").service(web::resource("").route(web::get().to(search_for_location))),
    );
}
