use actix_files::NamedFile;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    get,
    http::{
        header::{CACHE_CONTROL, EXPIRES},
        HeaderValue,
    },
    middleware::{Compress, Logger},
    web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};
use rustls::{
    internal::pemfile::{certs, pkcs8_private_keys},
    NoClientAuth, ServerConfig,
};
use std::{fs::File, io::BufReader, path::Path};

mod controllers;
mod services;
mod utils;

async fn create_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let pool: sqlx::PgPool = sqlx::pool::PoolOptions::new()
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not found"))
        .await?;

    Ok(pool)
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use dotenv::dotenv;

    dotenv().ok();

    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "actix_web=info,sqlx=debug");
        env_logger::init();
    }

    const HTTP_PORT: u32 = if cfg!(debug_assertions) { 8080 } else { 80 };
    const HTTPS_PORT: u32 = if cfg!(debug_assertions) { 8443 } else { 443 };
    let server_addr =
        std::env::var("SERVER_ADDR").expect("SERVER_ADDR variable not specified in .env file");
    let pool = create_pool().await.expect("Connection to database failed");

    // TLS configuration
    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(
        File::open(
            &std::env::var("CERTIFICATE_CHAIN_FILE")
                .expect("CERTIFICATE_CHAIN_FILE not found in variables environment"),
        )
        .unwrap(),
    );
    let key_file = &mut BufReader::new(
        File::open(
            &std::env::var("PRIVATE_KEY_FILE")
                .expect("PRIVATE_KEY_FILE not found in variables environment"),
        )
        .unwrap(),
    );
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    // Redirect HTTP to HTTPS
    let http = HttpServer::new(move || {
        App::new().wrap(utils::https::RedirectHTTPS::with_replacements(&[(
            "8080".to_owned(),
            "8443".to_owned(),
        )]))
    })
    .bind(&format!("{}:{}", server_addr, HTTP_PORT))?
    .run();

    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .secure(true),
            ))
            .service(controllers::index)
            .service(
                web::scope("/portfolio")
                    .service(controllers::portfolio::index)
                    .service(controllers::portfolio::get_project) // .service(controllers::portfolio::update_project)
                    .service(controllers::portfolio::insert_asset),
            )
            .service(controllers::motion_design)
            .service(controllers::contact::index)
            .service(controllers::contact::post)
            .service(
                web::scope("/user")
                    .service(controllers::user::login)
                    .service(controllers::user::logout),
            )
            .service(
                web::scope("/blog")
                    .service(controllers::blog::index)
                    .service(controllers::blog::show_category)
                    .service(controllers::blog::show_article),
            )
            .service(controllers::metrics::log)
            .service(serve_upload_file)
            .service(serve_public_file)
    })
    .bind_rustls(&format!("{}:{}", server_addr, HTTPS_PORT), config)
    .expect("Cannot bind openssl")
    .run();

    println!(
        "Server running at https://{}:{}",
        &std::env::var("SERVER_ADDR").unwrap(),
        HTTPS_PORT
    );

    futures::future::try_join(http, server)
        .await
        .expect("Error await");

    Ok(())
}
