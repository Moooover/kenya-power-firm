use actix_web::web;

pub mod search_locations;
pub mod subscribe_to_location;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/locations")
            .configure(search_locations::init_routes)
            .configure(subscribe_to_location::init_routes),
    );
}
