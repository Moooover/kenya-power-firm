mod authentication;
pub mod locations;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(authentication::init_routes)
            .configure(locations::init_routes),
    );
}
