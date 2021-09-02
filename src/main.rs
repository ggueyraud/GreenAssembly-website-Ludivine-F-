use actix_files::NamedFile;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware::Compress, web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod controllers;
mod services;
mod utils;

async fn create_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let pool: sqlx::PgPool = sqlx::pool::PoolOptions::new()
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not found"))
        .await?;

    Ok(pool)
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

    // Configure certificates
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).expect("SSL build");
    builder
        .set_private_key_file(
            &std::env::var("PRIVATE_KEY_FILE")
                .expect("PRIVATE_KEY_FILE not found in variables environment"),
            SslFiletype::PEM,
        )
        .expect("private key file not found");
    builder
        .set_certificate_chain_file(
            &std::env::var("CERTIFICATE_CHAIN_FILE")
                .expect("CERTIFICATE_CHAIN_FILE not found in variables environment"),
        )
        .expect("certificate chain file not found");

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
    })
    .bind_openssl(&format!("{}:{}", server_addr, HTTPS_PORT), builder)
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
