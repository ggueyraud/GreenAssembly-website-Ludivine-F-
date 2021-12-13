use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/portfolio")
                    .service(controllers::portfolio::insert_project)
                    .service(controllers::portfolio::delete_project)
                    .service(controllers::portfolio::create_category)
                    .service(controllers::portfolio::update_category)
                    .service(controllers::portfolio::delete_category)
                    .service(controllers::portfolio::get_project),
            )
            .service(
                web::scope("/blog")
                    .service(controllers::blog::get_category)
                    .service(controllers::blog::insert_category)
                    .service(controllers::blog::update_category)
                    .service(controllers::blog::delete_category)
                    .service(controllers::blog::get_article)
                    .service(controllers::blog::insert_article)
                    .service(controllers::blog::update_article)
                    .service(controllers::blog::delete_article),
            )
            .service(
                web::scope("/my_little_plus")
                    .service(controllers::admin::my_little_plus::edit_links)
                    .service(controllers::admin::my_little_plus::get_links),
            )
            .service(web::scope("/home").service(controllers::admin::home::edit_image)),
    );
}
