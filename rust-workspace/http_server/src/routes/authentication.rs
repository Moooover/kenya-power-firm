use crate::app_container::Application;
use crate::authentication::AuthenticatedUserInfo;
use crate::errors::ApiError;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use subscribers::contracts::create_or_update_subscriber::SubscriberInput;

#[derive(Serialize, Deserialize, Debug)]
struct UserRequest {
    name: String,
    email: String,
}

#[tracing::instrument(err, skip(app), level = "info")]
async fn authentication(
    user_details: web::Json<UserRequest>,
    app: web::Data<Application>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let _ = app
        .subscribers
        .create_or_update_subscriber(SubscriberInput {
            name: user_details.name.clone(),
            email: user_details.email.clone(),
            external_id: user.external_id.to_string(),
        })
        .await
        .map_err(ApiError::InternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/authenticate")
            .service(web::resource("").route(web::post().to(authentication))),
    );
}
