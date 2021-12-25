use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use sqlx::PgPool;

pub mod api;
pub mod admin;
pub mod blog;
pub mod metrics;
pub mod portfolio;
pub mod user;

#[get("/")]
pub async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "accueil").await {
        let mut token: Option<String> = None;

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
        }

        return Index {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/mes-petits-plus")]
async fn my_little_plus(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "mes-petits-plus").await {
        let (metric_id, links) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::my_little_plus::get_links(&pool)
        );

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = metric_id {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        let creations = match &links {
            Some(val) => {
                if let Some(link) = &val.creations {
                    if !link.is_empty() {
                        val.creations.clone()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };
        let shootings = match &links {
            Some(val) => {
                if let Some(link) = &val.shootings {
                    if !link.is_empty() {
                        val.shootings.clone()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        #[derive(Template)]
        #[template(path = "pages/my-little-plus.html")]
        struct MyLittlePlus {
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
            creations_link: Option<String>,
            shootings_link: Option<String>,
        }

        return MyLittlePlus {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
            creations_link: creations,
            shootings_link: shootings,
        }
        .into_response();
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

        #[derive(serde::Deserialize)]
        struct ChunkData {
            link: String,
        }

        if let Ok(chunk) = services::pages::chunks::get::<Chunk>(&pool, "content", "link").await {
            if let Ok(data) = serde_json::from_value::<ChunkData>(chunk.content) {
                let mut token: Option<String> = None;
                if let Ok(Some(id)) =
                    crate::controllers::metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id))
                        .await
                {
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
                }

                return MotionDesign {
                    title: page.title,
                    description: page.description,
                    year: chrono::Utc::now().year(),
                    metric_token: token,
                    link: data.link,
                }
                .into_response();
            }
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/contact")]
async fn contact(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "contact").await {
        let mut token: Option<String> = None;

        if let Ok(Some(id)) =
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await
        {
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
        }

        return Contact {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/mentions-legales")]
async fn legals(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "mentions-legales").await {
        let mut token: Option<String> = None;

        if let Ok(Some(id)) =
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await
        {
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
        }

        return Legals {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
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
