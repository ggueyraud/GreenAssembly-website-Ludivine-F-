use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/portfolio")
                    .service(controllers::api::portfolio::insert_project)
                    .service(controllers::api::portfolio::update_project)
                    .service(controllers::api::portfolio::delete_project)
                    .service(controllers::api::portfolio::create_category)
                    .service(controllers::api::portfolio::update_category)
                    .service(controllers::api::portfolio::delete_category)
                    .service(controllers::api::portfolio::get_project),
            )
            .service(
                web::scope("/blog")
                    .service(controllers::api::blog::get_category)
                    .service(controllers::api::blog::insert_category)
                    .service(controllers::api::blog::update_category)
                    .service(controllers::api::blog::delete_category)
                    .service(controllers::api::blog::get_article)
                    .service(controllers::api::blog::insert_article)
                    .service(controllers::api::blog::update_article)
                    .service(controllers::api::blog::delete_article),
            )
            .service(
                web::scope("/motion-design")
                    .service(controllers::api::update_motion_design_informations),
            )
            .service(
                web::scope("/my_little_plus")
                    .service(controllers::api::update_little_plus_informations),
            )
            .service(web::scope("/contact").service(controllers::api::contact))
            .service(web::scope("/home").service(controllers::api::update_home_informations))
            .service(web::scope("/settings").service(controllers::api::update_settings)),
    );
}
