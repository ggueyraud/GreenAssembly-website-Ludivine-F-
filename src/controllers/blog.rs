use super::metrics;
use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;

#[get("")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "blog").await {
        use slugmin::slugify;

        #[derive(sqlx::FromRow)]
        struct Category {
            id: i16,
            name: String,
            uri: String,
        }

        #[derive(sqlx::FromRow)]
        struct Article {
            id: i16,
            name: String,
            content: String,
            date: DateTime<Utc>,
        }

        let (metric_id, mut categories, articles) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::blog::categories::get_all::<Category>(
                &pool,
                r#"id, name, '' AS "uri""#,
                Some(true),
                Some(true)
            ),
            services::blog::articles::get_all1::<Article>(
                &pool,
                "ba.id, ba.name, ba.content, ba.date",
                None,
                None
            )
        );

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = metric_id {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        categories.iter_mut().for_each(|category| {
            category.uri = slugify(&format!("{}-{}", category.name, category.id))
        });

        #[derive(Template)]
        #[template(path = "blog_index.html")]
        struct Blog {
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
            categories: Vec<Category>,
            articles: Vec<Article>,
        }

        return Blog {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
            categories,
            articles,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}
