use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/blog")
            .service(controllers::blog::index)
            .service(controllers::blog::show_category)
            .service(controllers::blog::show_article),
    );
}
