use crate::controllers;
use actix_files::NamedFile;
use actix_web::{
    get,
    http::{
        header::{CACHE_CONTROL, EXPIRES},
        HeaderValue,
    },
    web, Error, HttpRequest, HttpResponse, Result,
};
use std::{fs::File, io::BufReader, path::Path};

pub mod admin;
pub mod api;
pub mod blog;
pub mod portfolio;
pub mod user;

fn serve_file(req: &HttpRequest, path: &Path, cache_duration: i64) -> Result<HttpResponse, Error> {
    match NamedFile::open(path) {
        Ok(file) => {
            use chrono::{Duration, Local};

            let mut response = file.into_response(&req)?;
            let now = Local::now() + Duration::days(cache_duration);
            let headers = response.headers_mut();
            headers.append(EXPIRES, HeaderValue::from_str(&now.to_rfc2822()).unwrap());
            headers.append(CACHE_CONTROL, HeaderValue::from_static("public"));

            Ok(response)
        }
        Err(_) => {
            use askama::Template;

            #[derive(Template)]
            #[template(path = "pages/404.html")]
            struct NotFound;

            Ok(HttpResponse::NotFound()
                .content_type("text/html")
                .body(NotFound.render().unwrap()))
        }
    }
}

#[get("/{filename:.*}")]
async fn serve_public_file(req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut file_path = format!("./public/{}", req.path());
    let path = if cfg!(debug_assertions) {
        let mut path = Path::new(&file_path);

        if !path.exists() {
            file_path = format!("./.build/development{}", req.path());
            path = Path::new(&file_path);
        }

        path
    } else {
        file_path = format!(".{}", req.path());
        Path::new(&file_path)
    };

    serve_file(&req, &path, 30)
}

#[get("/uploads/{filename:.*}")]
async fn serve_upload_file(req: HttpRequest) -> Result<HttpResponse, Error> {
    let file_path = format!(".{}", req.path());
    let path = Path::new(&file_path);
    serve_file(&req, &path, 30)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(controllers::index)
        .service(controllers::my_little_plus::index)
        .service(controllers::motion_design)
        .service(controllers::contact::index)
        .service(controllers::contact::post)
        .service(controllers::metrics::log)
        .service(serve_upload_file)
        .service(serve_public_file);
}
