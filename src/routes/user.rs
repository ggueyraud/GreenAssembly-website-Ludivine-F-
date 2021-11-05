use crate::controllers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user").service(
            web::scope("/portfolio")
                .service(controllers::user::login)
                .service(controllers::user::logout)
                .service(controllers::user::lost_password)
                .service(controllers::user::password_recovery),
        ),
    );
}
