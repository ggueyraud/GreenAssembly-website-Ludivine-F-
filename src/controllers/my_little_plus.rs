use super::metrics;
use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use sqlx::PgPool;

#[get("/mes-petits-plus")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
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
                    if link.as_str() != "" {
                        val.creations.clone()
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            None => None
        };
        let shootings = match &links {
            Some(val) => {
                if let Some(link) = &val.shootings {
                    if link.as_str() != "" {
                        val.shootings.clone()
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            None => None
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
