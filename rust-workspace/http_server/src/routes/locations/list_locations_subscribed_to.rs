use actix_web::{web, HttpRequest};
use itertools::Itertools;
use serde::Serialize;
use uuid::Uuid;

use crate::app_container::Application;
use crate::{authentication::AuthenticatedUserInfo, errors::ApiError};

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

#[tracing::instrument(err, skip(app), level = "info")]
async fn list_locations_subscribed_to(
    app: web::Data<Application>,
    req: HttpRequest,
) -> Result<web::Json<LocationsResponswWrapper>, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let subscriber = app
        .subscribers
        .authenticate(user.external_id.as_ref())
        .await
        .map_err(ApiError::InternalServerError)?;
    let locations = app
        .location_subscription
        .list_subscribed_locations(subscriber)
        .await
        .map_err(ApiError::InternalServerError)?;

    let response = LocationsResponswWrapper {
        items: locations
            .into_iter()
            .map(|location| LocationWithIdResponse {
                id: location.id.inner(),
                name: location.name.to_string(),
                address: location.address,
            })
            .collect_vec(),
    };

    Ok(web::Json(response))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/list/subscribed")
            .service(web::resource("").route(web::get().to(list_locations_subscribed_to))),
    );
}
