use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .service(controllers::admin::index)
            .service(controllers::admin::home_page)
            .service(controllers::admin::portfolio)
            .service(controllers::admin::my_little_plus_page)
            .service(controllers::admin::settings),
    );
}
