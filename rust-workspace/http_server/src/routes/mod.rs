mod authentication;
pub mod import_planned_power_interruptions;
pub mod locations;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(authentication::init_routes)
            .configure(locations::init_routes)
            .configure(import_planned_power_interruptions::init_routes),
    );
}
