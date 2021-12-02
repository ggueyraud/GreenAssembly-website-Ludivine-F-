use actix_extract_multipart::{File, Multipart};
use actix_identity::Identity;
use actix_web::{patch, Error, HttpResponse};
use serde::Deserialize;
use std::io::Write;
use webp::Encoder;

#[derive(Deserialize)]
pub struct HomeImage {
    pub image: File,
}

const IMG_PUBLIC_FOLDER: &str = "./public/img/";

#[patch("/image")]
async fn edit_image(id: Identity, data: Multipart<HomeImage>) -> Result<HttpResponse, Error> {
    if id.identity().is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let file_type = data.image.file_type();

    if !file_type.contains("jpeg") && !file_type.contains("png") && !file_type.contains("webp") {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let file_data = data.image.data();
    let image = image::load_from_memory(&file_data).unwrap();
    let name = "index";

    const MAX_MOBILE: (u32, u32) = (500, 500);
    const MAX_DESKTOP: (u32, u32) = (1000, 1000);

    // Create thumbnail
    image
        .thumbnail(MAX_MOBILE.0, MAX_MOBILE.1)
        .save_with_format(
            format!("{}{}_mobile.png", IMG_PUBLIC_FOLDER, name.clone()),
            image::ImageFormat::Png,
        )
        .unwrap();

    image
        .thumbnail(MAX_DESKTOP.0, MAX_DESKTOP.1)
        .save_with_format(
            format!("{}{}.png", IMG_PUBLIC_FOLDER, name.clone()),
            image::ImageFormat::Png,
        )
        .unwrap();

    // Create webp format
    let image_webp =
        Encoder::from_image(&image.resize(MAX_MOBILE.0, MAX_MOBILE.1, image::imageops::CatmullRom))
            .encode(100.0);
    let v = image_webp.iter().map(|a| *a).collect::<Vec<u8>>();
    let mut webp_file =
        std::fs::File::create(format!("{}{}_mobile.webp", IMG_PUBLIC_FOLDER, name)).unwrap();
    webp_file.write_all(&v).unwrap();

    let image_webp = Encoder::from_image(&image.resize(
        MAX_DESKTOP.0,
        MAX_DESKTOP.1,
        image::imageops::CatmullRom,
    ))
    .encode(100.0);
    let v = image_webp.iter().map(|a| *a).collect::<Vec<u8>>();
    let mut webp_file =
        std::fs::File::create(format!("{}{}.webp", IMG_PUBLIC_FOLDER, name)).unwrap();
    webp_file.write_all(&v).unwrap();

    Ok(HttpResponse::Ok().finish())
}
