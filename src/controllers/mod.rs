use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::PgPool;

pub mod admin;
pub mod api;
pub mod blog;
pub mod metrics;
pub mod portfolio;
pub mod user;

#[get("/")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "accueil").await {
        let mut token: Option<String> = None;

        match futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::settings::get(&pool)
        ) {
            (Ok(metric_id), Ok(settings)) => {
                if let Ok(Some(id)) =
                    metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await
                {
                    if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                        token = Some(metric_token.to_string());
                    }
                }
        
                #[derive(Template)]
                #[template(path = "pages/index.html")]
                struct Index {
                    title: String,
                    description: Option<String>,
                    year: i32,
                    metric_token: Option<String>,
                    settings: services::settings::Settings
                }
        
                return Index {
                    title: page.title,
                    description: page.description,
                    year: chrono::Utc::now().year(),
                    metric_token: token,
                    settings
                }
                .into_response();
            },
            _ => ()
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/mes-petits-plus")]
async fn my_little_plus(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "mes-petits-plus").await {
        #[derive(sqlx::FromRow)]
        struct Chunk {
            content: serde_json::Value,
        }

        #[derive(Deserialize)]
        struct ChunkData {
            value: Option<String>,
        }

        match futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::pages::chunks::get::<Chunk>(&pool, "content", "link_creations"),
            services::pages::chunks::get::<Chunk>(&pool, "content", "link_shootings"),
            services::settings::get(&pool)
        ) {
            (Ok(metric_id), Ok(creations), Ok(shootings), Ok(settings)) => {
                let creations = match serde_json::from_value::<ChunkData>(creations.content) {
                    Ok(data) => data.value,
                    Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
                };
                let shootings = match serde_json::from_value::<ChunkData>(shootings.content) {
                    Ok(data) => data.value,
                    Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
                };

                let mut token: Option<String> = None;
                if let Some(id) = metric_id {
                    if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                        token = Some(metric_token.to_string());
                    }
                }

                #[derive(Template)]
                #[template(path = "pages/my_little_plus.html")]
                struct MyLittlePlus {
                    title: String,
                    description: Option<String>,
                    year: i32,
                    metric_token: Option<String>,
                    creations_link: Option<String>,
                    shootings_link: Option<String>,
                    settings: services::settings::Settings
                }

                return MyLittlePlus {
                    title: page.title,
                    description: page.description,
                    year: chrono::Utc::now().year(),
                    metric_token: token,
                    creations_link: creations,
                    shootings_link: shootings,
                    settings
                }
                .into_response();
            }
            _ => return Ok(HttpResponse::InternalServerError().finish()),
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/motion-design")]
async fn motion_design(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "motion-design").await {
        #[derive(sqlx::FromRow)]
        struct Chunk {
            content: serde_json::Value,
        }

        #[derive(Deserialize)]
        struct ChunkData {
            link: String,
        }

        match futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::pages::chunks::get::<Chunk>(&pool, "content", "link"),
            services::settings::get(&pool)
        ) {
            (Ok(metric_id), Ok(chunk), Ok(settings)) => {
                if let Ok(data) = serde_json::from_value::<ChunkData>(chunk.content) {
                    let mut token: Option<String> = None;

                    if let Some(id) = metric_id {
                        if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                            token = Some(metric_token.to_string());
                        }
                    }

                    #[derive(Template)]
                    #[template(path = "pages/motion_design.html")]
                    struct MotionDesign {
                        title: String,
                        description: Option<String>,
                        year: i32,
                        metric_token: Option<String>,
                        link: String,
                        settings: services::settings::Settings
                    }
            
                    return MotionDesign {
                        title: page.title,
                        description: page.description,
                        year: chrono::Utc::now().year(),
                        metric_token: token,
                        link: data.link,
                        settings
                    }
                    .into_response();
                }

            },
            _ => ()
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/contact")]
async fn contact(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "contact").await {
        if let (Ok(metric_id), Ok(settings)) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::settings::get(&pool)
        ) {
            let mut token: Option<String> = None;
            if let Some(id) = metric_id {
                if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                    token = Some(metric_token.to_string());
                }
            }

            #[derive(Template)]
            #[template(path = "pages/contact.html")]
            struct Contact {
                title: String,
                description: Option<String>,
                year: i32,
                metric_token: Option<String>,
                settings: services::settings::Settings
            }
    
            return Contact {
                title: page.title,
                description: page.description,
                year: chrono::Utc::now().year(),
                metric_token: token,
                settings
            }
            .into_response();
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/mentions-legales")]
async fn legals(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "mentions-legales").await {
        if let (Ok(metric_id), Ok(settings)) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::settings::get(&pool)
        ) {
            let mut token: Option<String> = None;
            if let Some(id) = metric_id {
                if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                    token = Some(metric_token.to_string());
                }
            }

            #[derive(Template)]
            #[template(path = "pages/legals.html")]
            struct Legals {
                title: String,
                description: Option<String>,
                year: i32,
                metric_token: Option<String>,
                settings: services::settings::Settings
            }
    
            return Legals {
                title: page.title,
                description: page.description,
                year: chrono::Utc::now().year(),
                metric_token: token,
                settings
            }
            .into_response();
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
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

    #[actix_rt::test]
    async fn test_motion_design() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app =
            test::init_service(App::new().data(pool.clone()).service(super::motion_design)).await;
        let resp = test::TestRequest::get()
            .uri("/motion-design")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    // #[actix_rt::test]
    // async fn test_index() {
    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();
    //     let mut app = test::init_service(App::new().data(pool.clone()).service(super::index)).await;
    //     let resp = test::TestRequest::get()
    //         .uri("/")
    //         .send_request(&mut app)
    //         .await;

    //     assert!(resp.status().is_success());
    // }
}
