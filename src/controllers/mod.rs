use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use sqlx::PgPool;

pub mod contact;
pub mod metrics;
pub mod portfolio;
pub mod user;

#[get("/")]
pub async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "accueil").await {
        metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await;

        #[derive(Template)]
        #[template(path = "index.html")]
        struct Index {
            title: String,
            description: Option<String>,
            year: i32,
        }

        return Index {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/motion-design")]
async fn motion_design(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "motion-design").await {
        let (_, videos) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::videos::get_all(&pool)
        );

        #[derive(Template)]
        #[template(path = "motion_design.html")]
        struct MotionDesign {
            videos: Vec<services::videos::Video>,
            title: String,
            description: Option<String>,
            year: i32,
        }

        return MotionDesign {
            videos,
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
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
}
