use actix_extract_multipart::{File, Multipart};
use actix_identity::Identity;
use actix_web::{patch, Error, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HomeImage {
    pub image: File,
}

#[patch("/image")]
async fn edit_image(session: Identity, data: Multipart<HomeImage>) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish()
    }

    let mut uploader = crate::utils::image::Uploader::new();

    if !&["image/jpeg", "image/png", "image/webp"].contains(&data.image.file_type().as_str()) {
        return HttpResponse::BadRequest().finish()
    }

    match image::load_from_memory(data.image.data()) {
        Ok(image) => {
            if uploader.handle(&image, "index", None, Some((1000, 1000))).is_err() {
                return HttpResponse::BadRequest().finish()
            }
        },
        Err(_) => return HttpResponse::InternalServerError().finish()
    }

    uploader.clear();

    HttpResponse::Ok().finish()
}
