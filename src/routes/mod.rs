use crate::controllers;
use actix_web::web;

pub mod admin;
pub mod api;
pub mod blog;
pub mod portfolio;
pub mod user;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(controllers::index)
        .service(controllers::my_little_plus::index)
        .service(controllers::motion_design)
        .service(controllers::legals)
        .service(controllers::contact::index)
        .service(controllers::contact::post)
        .service(controllers::metrics::log);
}
