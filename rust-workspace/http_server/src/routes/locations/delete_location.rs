use crate::{
    authentication::AuthenticatedUserInfo, errors::ApiError,
    use_case_app_container::UseCaseAppContainer,
};
use actix_web::{web, HttpRequest, HttpResponse};
use itertools::Itertools;
use serde::Serialize;
use use_cases::subscriber_locations::data::{AdjuscentLocation, LocationWithId};
use uuid::Uuid;

async fn delete_primary_location(
    id: web::Path<Uuid>,
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let interactor = app.get_client().delete_subscribed_location();
    interactor
        .delete_primary_location(&user, id.into_inner().into())
        .await
        .map_err(ApiError::InternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

async fn delete_adjuscent_location(
    id: web::Path<Uuid>,
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;
    let interactor = app.get_client().delete_subscribed_location();
    interactor
        .delete_adjuscent_location(&user, id.into_inner().into())
        .await
        .map_err(ApiError::InternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(
                web::resource("/primary_location/{id}")
                    .route(web::delete().to(delete_primary_location)),
            )
            .service(
                web::resource("/adjuscent_location/{id}")
                    .route(web::delete().to(delete_adjuscent_location)),
            ),
    );
}
