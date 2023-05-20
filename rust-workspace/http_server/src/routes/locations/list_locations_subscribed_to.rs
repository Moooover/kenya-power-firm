use actix_web::{web, HttpRequest};
use itertools::Itertools;
use serde::Serialize;
use use_cases::subscriber_locations::list_subscribed_locations::LocationWithId;
use uuid::Uuid;

use crate::{
    authentication::AuthenticatedUserInfo, errors::ApiError,
    use_case_app_container::UseCaseAppContainer,
};

#[derive(Serialize)]
struct LocationWithIdResponse {
    id: Uuid,
    name: String,
    address: String,
}

#[derive(Serialize)]
struct LocationsResponswWrapper {
    items: Vec<LocationWithIdResponse>,
}

impl From<LocationWithId> for LocationWithIdResponse {
    fn from(value: LocationWithId) -> Self {
        Self {
            id: value.id.inner(),
            name: value.name,
            address: value.address,
        }
    }
}

#[tracing::instrument(err, skip(app), level = "info")]
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
