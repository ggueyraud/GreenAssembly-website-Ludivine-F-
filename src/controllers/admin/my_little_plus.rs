use crate::services;
use actix_identity::Identity;
use actix_web::{get, patch, web, Error, HttpResponse};
use sqlx::PgPool;

#[get("/links")]
async fn get_links(pool: web::Data<PgPool>, id: Identity) -> Result<HttpResponse, Error> {
    if id.identity().is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    match services::my_little_plus::get_links(&pool).await {
        Some(val) => Ok(HttpResponse::Ok().json(val)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[patch("/links")]
async fn edit_links(
    pool: web::Data<PgPool>,
    id: Identity,
    links: web::Json<services::my_little_plus::Links>,
) -> Result<HttpResponse, Error> {
    if id.identity().is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    use regex::Regex;
    let http_regex = Regex::new(r"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$").unwrap();

    if let Some(url) = &links.creations {
        if url != "" && !http_regex.is_match(url) {
            return Ok(HttpResponse::BadRequest().finish());
        }
    }
    if let Some(url) = &links.shootings {
        if url != "" && !http_regex.is_match(url) {
            return Ok(HttpResponse::BadRequest().finish());
        }
    }

    match services::my_little_plus::edit_links(&pool, &links).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
