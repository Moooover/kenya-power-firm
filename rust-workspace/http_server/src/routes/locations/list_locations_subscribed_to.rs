use actix_web::{web, HttpRequest};
use itertools::Itertools;
use serde::Serialize;
use use_cases::subscriber_locations::data::{AdjuscentLocation, LocationWithId};
use uuid::Uuid;

use crate::{
    authentication::AuthenticatedUserInfo, errors::ApiError,
    use_case_app_container::UseCaseAppContainer,
};

#[derive(Serialize)]
struct AdjuscentLocationResponse {
    id: Uuid,
    name: String,
    address: String,
}

#[derive(Serialize)]
struct LocationWithIdResponse {
    id: Uuid,
    name: String,
    address: String,
    adjuscent_locations: Vec<AdjuscentLocationResponse>,
}

#[derive(Serialize)]
struct LocationsResponswWrapper {
    items: Vec<LocationWithIdResponse>,
}

impl From<AdjuscentLocation> for AdjuscentLocationResponse {
    fn from(value: AdjuscentLocation) -> Self {
        AdjuscentLocationResponse {
            id: value.id.inner(),
            name: value.name,
            address: value.address,
        }
    }
}

impl From<LocationWithId> for LocationWithIdResponse {
    fn from(value: LocationWithId) -> Self {
        Self {
            id: value.id.inner(),
            name: value.name,
            address: value.address,
            adjuscent_locations: value
                .adjuscent_locations
                .into_iter()
                .map_into()
                .collect_vec(),
        }
    }
}

async fn list_locations_subscribed_to(
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<web::Json<LocationsResponswWrapper>, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let interactor = app.get_client().list_locations_subcribed_to();
    let results = interactor
        .list(&user)
        .await
        .map_err(ApiError::InternalServerError)?;

    let response = LocationsResponswWrapper {
        items: results.into_iter().map_into().collect_vec(),
    };

    Ok(web::Json(response))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/list/subscribed")
            .service(web::resource("").route(web::get().to(list_locations_subscribed_to))),
    );
}
