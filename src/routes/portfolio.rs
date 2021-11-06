use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/portfolio")
            .service(controllers::portfolio::index)
            .service(controllers::portfolio::view_project),
    );
}
