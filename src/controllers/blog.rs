use super::metrics;
use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct Article {
    name: String,
    uri: String,
    content: String,
    date: String,
    international_date: String,
    cover: String,
}

#[derive(sqlx::FromRow)]
struct Category {
    name: String,
    uri: String,
}

#[get("")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "blog").await {
        let (metric_id, categories, articles) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::blog::categories::get_all::<Category>(
                &pool,
                "name, uri",
                Some(true),
                Some(true)
            ),
            services::blog::articles::get_all1::<Article>(
                &pool,
                r#"ba.name,
                ba.uri,
                CASE 
                    WHEN LENGTH(ba.content) > 300 THEN
                        SUBSTR(ba.content, 0, 300) || '...'
                    ELSE
                        SUBSTR(ba.content, 0, 300)
                END AS CONTENT,
                TO_CHAR(ba.date, 'DD/MM/YYYY') AS "date",
                TO_CHAR(ba.date, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS international_date,
                f.path AS cover"#,
                Some(true),
                Some(true),
                None
            )
        );

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = metric_id {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        #[derive(Template)]
        #[template(path = "blog/index.html")]
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

#[get("/categories/{name}-{id}")]
async fn show_category(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    web::Path((name, id)): web::Path<(String, i16)>,
) -> Result<HttpResponse, Error> {
    if !services::blog::categories::exists_for_uri(&pool, &format!("{}-{}", name, id)).await {
        return Ok(HttpResponse::NotFound().finish());
    }

    #[derive(sqlx::FromRow)]
    struct CategoryDetails {
        name: String,
        description: Option<String>,
    }

    let (metric_id, category, categories, articles) = futures::join!(
        metrics::add(&pool, &req, services::metrics::BelongsTo::BlogPost(id)),
        services::blog::categories::get::<CategoryDetails>(&pool, "name, description", id),
        services::blog::categories::get_all::<Category>(&pool, "name, uri", Some(true), Some(true)),
        services::blog::articles::get_all1::<Article>(
            &pool,
            r#"ba.name,
            ba.uri,
            CASE 
                WHEN LENGTH(ba.content) > 300 THEN
                    SUBSTR(ba.content, 0, 300) || '...'
                ELSE
                    SUBSTR(ba.content, 0, 300)
            END AS CONTENT,
            TO_CHAR(ba.date, 'DD/MM/YYYY') AS "date",
            TO_CHAR(ba.date, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS international_date,
            f.path AS cover"#,
            Some(true),
            Some(true),
            Some(id)
        )
    );

    // TODO : see with Vincent to refactor this behavior
    let category = category.unwrap();

    let mut token: Option<String> = None;
    if let Ok(Some(id)) = metric_id {
        if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
            token = Some(metric_token.to_string());
        }
    }

    #[derive(Template)]
    #[template(path = "blog/category.html")]
    struct BlogCategory {
        title: String,
        description: Option<String>,
        year: i32,
        metric_token: Option<String>,
        categories: Vec<Category>,
        articles: Vec<Article>,
    }

    BlogCategory {
        title: category.name,
        description: category.description,
        year: chrono::Utc::now().year(),
        metric_token: token,
        categories,
        articles,
    }
    .into_response()
}

#[cfg(test)]
mod tests {
    use crate::create_pool;
    use crate::controllers;
    use actix_web::{test, App, web, http::StatusCode};
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_index() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/blog").service(controllers::blog::index)),
        )
        .await;
        let res = test::TestRequest::get()
            .uri("/blog")
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/blog").service(controllers::blog::show_category)),
        )
        .await;
        let res = test::TestRequest::get()
            .uri("/blog/categories/print-1")
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_category_doesnt_exist() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/blog").service(controllers::blog::show_category)),
        )
        .await;
        let res = test::TestRequest::get()
            .uri("/blog/print-11")
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
