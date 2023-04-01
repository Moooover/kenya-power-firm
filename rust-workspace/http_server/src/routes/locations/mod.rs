use actix_web::web;

mod list_locations_subscribed_to;
pub mod search_locations;
pub mod subscribe_to_location;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/locations")
            .configure(search_locations::init_routes)
            .configure(subscribe_to_location::init_routes)
            .configure(list_locations_subscribed_to::init_routes),
    );
}
