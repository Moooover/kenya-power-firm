use actix_web::{web, HttpRequest, HttpResponse};
use entities::locations::ExternalLocationId;
use serde::Deserialize;
use use_cases::subscriber_locations::data::LocationInput;

use crate::{
    authentication::AuthenticatedUserInfo, errors::ApiError,
    use_case_app_container::UseCaseAppContainer,
};

#[derive(Deserialize)]
struct LocationSubscriptionRequest {
    location: String,
    nearby_locations: Vec<String>,
}

impl From<LocationSubscriptionRequest> for LocationInput<ExternalLocationId> {
    fn from(value: LocationSubscriptionRequest) -> Self {
        let location = value.location.into();
        let nearby_locations = value
            .nearby_locations
            .into_iter()
            .map(|location| location.into())
            .collect::<Vec<_>>();

        LocationInput {
            id: location,
            nearby_locations,
        }
    }
}

async fn subscribe_to_location(
    data: web::Json<LocationSubscriptionRequest>,
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let interactor = app.get_client().subscribe_to_location();
    let user: AuthenticatedUserInfo = (&req).try_into()?;

    let data = data.into_inner();
    interactor
        .subscribe(&user, data.into())
        .await
        .map_err(ApiError::InternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subscribe")
            .service(web::resource("").route(web::post().to(subscribe_to_location))),
    );
}
