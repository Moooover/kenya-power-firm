use std::collections::HashMap;

use crate::authentication::AuthenticatedUserInfo;
use crate::errors::ApiError;
use crate::use_case_app_container::UseCaseAppContainer;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use url::Url;
use use_cases::import_affected_areas::{Area, County, ImportInput, Region};

#[derive(Serialize, Deserialize)]
struct AreaRequest {
    name: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    locations: Vec<String>,
}

#[derive(Serialize, Deserialize)]

struct CountyRequest {
    name: String,
    areas: Vec<AreaRequest>,
}

#[derive(Serialize, Deserialize)]

struct RegionRequest {
    name: String,
    counties: Vec<CountyRequest>,
}

#[derive(Serialize, Deserialize)]
struct ImportData {
    url: Url,
    regions: Vec<RegionRequest>,
}

async fn import(
    import_data: web::Json<ImportData>,
    app: web::Data<UseCaseAppContainer>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user: AuthenticatedUserInfo = (&req).try_into()?;

    let interactor = app.get_client().import_planned_blackouts();
    let import_data = import_data.into_inner();
    let regions = import_data.regions.into_iter().map_into().collect_vec();
    let data = HashMap::from_iter([(import_data.url, regions)]);
    let data = ImportInput(data);
    interactor
        .import(&user, data)
        .await
        .map_err(ApiError::InternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/import-planned-power-interruptions")
            .service(web::resource("").route(web::post().to(import))),
    );
}

impl From<RegionRequest> for Region {
    fn from(value: RegionRequest) -> Self {
        Self {
            name: value.name,
            counties: value.counties.into_iter().map(Into::into).collect_vec(),
        }
    }
}

impl From<CountyRequest> for County {
    fn from(value: CountyRequest) -> Self {
        Self {
            name: value.name,
            areas: value.areas.into_iter().map(Into::into).collect_vec(),
        }
    }
}

impl From<AreaRequest> for Area {
    fn from(value: AreaRequest) -> Self {
        Self {
            name: value.name,
            from: value.from.into(),
            to: value.to.into(),
            locations: value.locations,
        }
    }
}
