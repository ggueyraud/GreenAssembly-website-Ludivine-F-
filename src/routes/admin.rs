use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user").service(
            web::scope("/portfolio")
                .service(controllers::admin::index)
                .service(controllers::admin::portfolio)
                .service(controllers::admin::settings),
        ),
    );
}
