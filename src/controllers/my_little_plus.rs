use super::metrics;
use crate::services;
use actix_web::{get, patch, web, Error, HttpRequest, HttpResponse};
use actix_identity::Identity;
use actix_extract_multipart::*;
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
struct PatchImage {
    image: Option<File>, // Image to upload (create a new row in files SQL table)
    file_id: Option<i32> // ID of the file in the files SQL table, if we want use existing file
}

const IMG_PATH: &str = "public/img/";

#[get("/mes-petits-plus")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "mes-petits-plus").await {
        let (metric_id, images) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::my_little_plus::get_images(&pool)
        );

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = metric_id {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        #[derive(Template)]
        #[template(path = "pages/my-little-plus.html")]
        struct MyLittlePlus {
            images: Vec<String>, // String path
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
        }

        // let images = vec![
        //     "/img/Peinture_1.jpg".to_owned(),
        //     "/img/Peinture_2.jpg".to_owned(),
        //     "/img/Peinture_3.jpg".to_owned(),
        //     "/img/OISEAU_1.jpg".to_owned(),
        //     "/img/PHOTO_01.jpg".to_owned(),
        //     "/img/PHOTO_3.jpg".to_owned(),
        //     "/img/HOTO_02.jpg".to_owned()
        // ];

        return MyLittlePlus {
            images,
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

async fn save_image(pool: &PgPool, image: &File, path: &str) -> Result<i32, HttpResponse> {
    println!("On sauvegarde l'image");

    let extension = match image.name().rsplit_once('.') {
        Some(ext) => ext.1,
        None => return Err(HttpResponse::BadRequest()
        .json(format!("File don't have any extension")))
    };
    let name = image.name().strip_suffix(extension);
    let path = format!("{}{}", path, image.name());
    // If bdd insertion failed, we delete the file
    match services::files::insert(pool, name, Some(path.as_str())).await {
        Ok(f_id) => Ok(f_id),
        Err(_) => {
            delete_image_file(image.name(), IMG_PATH);

            return Err(HttpResponse::InternalServerError()
                .json(format!("Bdd insertion failed\"{}\"", image.name())))
        }
    }
}
fn delete_image_file(filename: &String, path: &str) {
    println!("On supprime l'image")
}

// id of the image to update(in my_little_plus_images), 1 to 7
#[patch("/mes-petits-plus/image/{id}")]
async fn patch_image(
    pool: web::Data<PgPool>,
    session: Identity,
    id: web::Path<i16>,
    payload: actix_multipart::Multipart,
) -> Result<HttpResponse, Error> {
    if session.identity().is_none() {
        return Ok(HttpResponse::Unauthorized().finish())
    }

    let id = id.into_inner();

    let form_data = match extract_multipart::<PatchImage>(payload).await {
        Ok(data) => data,
        Err(_) => return Ok(HttpResponse::BadRequest().json("The data received does not correspond to those expected"))
    };

    let image_file = form_data.image;
    let mut file_id = form_data.file_id;

    if (image_file.is_none() && file_id.is_none()) || 
       (image_file.is_some() && file_id.is_some())
    {
        return Ok(HttpResponse::BadRequest().json("You must send an image or the file id"))
    }

    if let Some(img_data) = image_file {
        file_id = match save_image(&pool, &img_data, IMG_PATH).await {
            Ok(f_id) => Some(f_id),
            Err(e) => return Ok(e)
        };

    }

    if !services::my_little_plus::patch_image(&pool, id, file_id.unwrap()).await {
        return Ok(HttpResponse::InternalServerError().json("Image update failed"))
    }

    Ok(HttpResponse::Ok().finish())
}


#[cfg(test)]
mod tests {
    use crate::create_pool;
    use actix_web::{test, App};
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_index() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(super::index)).await;
        let resp = test::TestRequest::get()
            .uri("/")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }
}