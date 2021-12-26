use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .service(controllers::admin::index)
            .service(controllers::admin::home_page)
            .service(controllers::admin::portfolio)
            .service(controllers::admin::motion_design)
            .service(controllers::admin::my_little_plus)
            .service(controllers::admin::settings)
            .service(controllers::admin::index)
            .service(controllers::admin::blog),
    );
}
