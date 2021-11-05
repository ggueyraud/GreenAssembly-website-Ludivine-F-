use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    middleware::{Compress, Logger},
    App, HttpServer, Result,
};
use rustls::{
    internal::pemfile::{certs, pkcs8_private_keys},
    NoClientAuth, ServerConfig,
};
use std::{fs::File, io::BufReader};

mod controllers;
mod routes;
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
            .configure(routes::config)
            .configure(routes::api::config)
            .configure(routes::portfolio::config)
            .configure(routes::blog::config)
            .configure(routes::user::config)
            .configure(routes::admin::config)
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
