use crate::errors::ApiError;
use crate::use_case_app_container::UseCaseAppContainer;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use use_cases::authentication::User;

#[derive(Serialize, Deserialize)]
struct UserRequest {
    name: String,
    email: String,
    external_id: String,
}

async fn authentication(
    user_details: web::Json<UserRequest>,
    app: web::Data<UseCaseAppContainer>,
) -> Result<HttpResponse, ApiError> {
    let UserRequest {
        name,
        email,
        external_id,
    } = user_details.into_inner();
    let auth_interactor = app.get_client().authentication();
    auth_interactor
        .authenticate(User {
            name,
            email,
            external_id,
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
