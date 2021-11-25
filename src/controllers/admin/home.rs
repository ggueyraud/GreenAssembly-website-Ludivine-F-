use actix_identity::Identity;
use actix_web::{patch, web, Error, HttpResponse};
use sqlx::PgPool;
use serde::{Deserialize};
use actix_extract_multipart::{File, Multipart};

#[derive(Deserialize)]
pub struct HomeImage {
    pub image: File,
}

#[patch("/image")]
async fn edit_image(pool: web::Data<PgPool>, id: Identity, data: Multipart<HomeImage>)
-> Result<HttpResponse, Error> {
    if id.identity().is_none() {
        return Ok(HttpResponse::Forbidden().finish())
    }

    println!("Image name: {}", &data.image.name());

    Ok(HttpResponse::Ok().finish())
}