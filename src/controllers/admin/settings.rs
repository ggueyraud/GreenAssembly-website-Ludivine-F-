use crate::utils::{image::Uploader, patch::Patch};
use actix_extract_multipart::{File, Multipart};
use actix_identity::Identity;
use actix_web::{get, patch, web, Error, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use sqlx::PgPool;

#[get("/parametres")]
pub async fn index(session: Identity, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if session.identity().is_none() {
        return Ok(HttpResponse::Found().header("location", "/admin").finish());
    }

    #[derive(Template)]
    #[template(path = "pages/admin/settings.html")]
    struct Setting {
        // categories: Vec<services::projects::Category>,
    // projects: Vec<services::projects::Project>,
    }

    return Setting {
        // categories,
        // projects,
    }
    .into_response();
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateParametersForm {
    #[serde(skip_serializing)]
    logo: Patch<File>,
    #[serde(skip_serializing)]
    favicon: Patch<File>,
    background_color: Patch<String>,
    title_color: Patch<String>,
    text_color: Patch<String>,
}

#[patch("")]
pub async fn update(
    session: Identity,
    pool: web::Data<PgPool>,
    form: Multipart<UpdateParametersForm>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let mut uploader = Uploader::new();

    if let Patch::Value(logo) = &form.logo {
        match image::load_from_memory(logo.data()) {
            Ok(image) => {
                // if uploader
                //     .handle(&image, "logo", Some(()))
                //     .is_err() {
                //         return HttpResponse::BadRequest().finish()
                //     }
            }
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }

    if let Patch::Value(favicon) = &form.favicon {
        match image::load_from_memory(favicon.data()) {
            Ok(favicon) => {
                // if uploader
                //     .handle(&favicon, "favicon", Some((64, 64)), Some(()))
                //     .is_err() {
                //         return HttpResponse::BadRequest().finish()
                //     }
            }
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }

    let mut fields_need_update = crate::utils::patch::extract_fields(&*form);

    HttpResponse::Ok().finish()
}
