use crate::{authentication::AuthenticatedUserInfo, errors::ApiError};
use actix_web::{web, HttpRequest, HttpResponse};

use crate::app_container::Application;
use uuid::Uuid;

#[tracing::instrument(err, skip(app), level = "info")]
async fn delete_primary_location(
    id: web::Path<Uuid>,
    app: web::Data<Application>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let subscriber_id = app
        .subscribers
        .authenticate(&user.external_id.as_ref())
        .await
        .map_err(ApiError::InternalServerError)?;
    let _ = app
        .location_subscription
        .unsubscribe_from_location(subscriber_id, id.into_inner().into())
        .await;

    Ok(HttpResponse::Ok().finish())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("").service(
        web::resource("/primary_location/{id}").route(web::delete().to(delete_primary_location)),
    ));
}
