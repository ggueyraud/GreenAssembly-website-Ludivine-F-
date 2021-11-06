use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api").service(
            web::scope("/portfolio")
                .service(controllers::portfolio::insert_project)
                .service(controllers::portfolio::delete_project)
                .service(controllers::portfolio::create_category)
                .service(controllers::portfolio::update_category)
                .service(controllers::portfolio::delete_category)
                .service(controllers::portfolio::get_project)
        ),
    );
}
