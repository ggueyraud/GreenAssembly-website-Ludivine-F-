use crate::controllers;
use actix_web::web;

pub mod admin;
pub mod api;
pub mod user;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(controllers::index)
        .service(controllers::my_little_plus)
        .service(controllers::motion_design)
        .service(controllers::legals)
        .service(controllers::contact)
        .service(controllers::metrics::log)
        .service(
            web::scope("/portfolio")
            .service(controllers::portfolio::index)
            .service(controllers::portfolio::view_project)
        )
        .service(
            web::scope("/blog")
                .service(controllers::blog::index)
                .service(controllers::blog::show_category)
                .service(controllers::blog::show_article),
        );
}
