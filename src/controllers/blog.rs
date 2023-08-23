use super::metrics;
use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use sqlx::{FromRow, PgPool};

#[derive(FromRow)]
struct Article {
    title: String,
    uri: String,
    description: Option<String>,
    date: String,
    international_date: String,
    cover: String,
}

#[derive(FromRow)]
struct Category {
    name: String,
    uri: String,
}

#[derive(sqlx::FromRow)]
struct Page {
    id: i16,
    title: String,
    description: Option<String>,
}

#[get("")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get::<Page>(&pool, "id, title, description", "/blog").await {
        let (metric_id, categories, articles, settings) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::blog::categories::get_all::<Category>(&pool, "name, uri", Some(true), None),
            services::blog::articles::get_all::<Article>(
                &pool,
                r#"ba.title,
                ba.uri,
                ba.description,
                TO_CHAR(ba.date, 'DD/MM/YYYY') AS "date",
                TO_CHAR(ba.date, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS international_date,
                f.path AS cover"#,
                Some(true),
                None,
                None
            ),
            services::settings::get(&pool)
        );

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = metric_id {
            token = Some(id.to_string());
        }

        #[derive(Template)]
        #[template(path = "pages/blog/index.html")]
        struct Blog {
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
            categories: Vec<Category>,
            articles: Vec<Article>,
            settings: services::settings::Settings
        }

        return Blog {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
            categories,
            articles,
            settings: settings.unwrap()
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

    #[derive(FromRow)]
    struct CategoryDetails {
        name: String,
        description: Option<String>,
        is_seo: Option<bool>,
    }

    let (metric_id, category, categories, articles, settings) = futures::join!(
        metrics::add(&pool, &req, services::metrics::BelongsTo::BlogPost(id)),
        services::blog::categories::get::<CategoryDetails>(&pool, "name, description, is_seo", id),
        services::blog::categories::get_all::<Category>(&pool, "name, uri", Some(true), None),
        services::blog::articles::get_all::<Article>(
            &pool,
            r#"ba.title,
            ba.uri,
            ba.description,
            TO_CHAR(ba.date, 'DD/MM/YYYY') AS "date",
            TO_CHAR(ba.date, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS international_date,
            f.path AS cover"#,
            Some(true),
            None,
            Some(id)
        ),
        services::settings::get(&pool)
    );

    // TODO : see with Vincent to refactor this behavior
    let category = category.unwrap();

    let mut token: Option<String> = None;
    if let Ok(Some(id)) = metric_id {
        token = Some(id.to_string());
    }

    #[derive(Template)]
    #[template(path = "pages/blog/category.html")]
    struct BlogCategory {
        title: String,
        description: Option<String>,
        is_seo: Option<bool>,
        year: i32,
        metric_token: Option<String>,
        categories: Vec<Category>,
        articles: Vec<Article>,
        settings: services::settings::Settings
    }

    BlogCategory {
        title: category.name,
        description: category.description,
        is_seo: category.is_seo,
        year: chrono::Utc::now().year(),
        metric_token: token,
        categories,
        articles,
        settings: settings.unwrap()
    }
    .into_response()
}

#[get("/articles/{name}-{id}")]
async fn show_article(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    web::Path((name, id)): web::Path<(String, i16)>,
) -> Result<HttpResponse, Error> {
    if !services::blog::articles::exists_for_uri(&pool, &format!("{}-{}", name, id)).await {
        return Ok(HttpResponse::NotFound().finish());
    }

    #[derive(FromRow, Debug)]
    struct Article {
        title: String,
        category_id: Option<i16>,
        cover_path: String,
        description: Option<String>,
        content: String,
        date: String,
        international_date: String,
        // As international date format
        modified_date: Option<String>,
        is_published: Option<bool>,
        is_seo: Option<bool>,
    }

    match services::blog::articles::get::<Article>(
        &pool,
        r#"title,
    category_id,
    f.path AS cover_path,
    description,
    content,
    TO_CHAR(date, 'DD/MM/YYYY') AS "date",
    TO_CHAR(date, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS international_date,
    CASE
        WHEN modified_date IS NOT NULL
            THEN 
                TO_CHAR(modified_date, 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
        ELSE NULL
    END AS modified_date,
    is_published,
    is_seo"#,
        id,
    )
    .await
    {
        Ok(mut article) => {
            match article.is_published {
                None | Some(false) => return Ok(HttpResponse::NotFound().finish()),
                _ => (),
            }

            #[derive(Template)]
            #[template(path = "pages/blog/article.html")]
            struct BlogArticle {
                article: Article,
                category: Option<Category>,
                categories: Vec<Category>,
                year: i32,
                metric_token: Option<String>,
                settings: services::settings::Settings
            }

            #[derive(FromRow)]
            struct Category {
                name: String,
                uri: String,
            }

            let mut category = Option::<Category>::None;

            if let Some(category_id) = article.category_id {
                category = Some(
                    services::blog::categories::get::<Category>(&pool, "name, uri", category_id)
                        .await
                        .unwrap(),
                );
            }

            let (metric_id, images, categories, settings) = futures::join!(
                metrics::add(&pool, &req, services::metrics::BelongsTo::BlogPost(id)),
                services::blog::articles::images::get_all(&pool, id),
                services::blog::categories::get_all::<Category>(
                    &pool,
                    "name, uri",
                    Some(true),
                    None
                ),
                services::settings::get(&pool)
            );

            for image in &images {
                let filename = image.path.split('.').collect::<Vec<_>>();
                let filename = filename.get(0).unwrap();

                article.content = article.content.replacen(
                    &format!("[[{}]]", image.id),
                    &format!(
                        r#"<picture class="lazy">
                            <source data-srcset="/uploads/mobile/{}.webp" media="(max-width: 768px)" type="image/webp" />
                            <source data-srcset="/uploads/mobile/{}" media="(max-width: 768px)" />
                            <source data-srcset="/uploads/{}.webp" media="(min-width: 768px)" type="image/webp" />

                            <img data-src="/uploads/{}" />
                        </picture>"#,
                        filename,
                        image.path,
                        filename,
                        image.path
                    ),
                    1
                );
            }

            let mut token: Option<String> = None;
            if let Ok(Some(id)) = metric_id {
                token = Some(id.to_string());
            }

            BlogArticle {
                article,
                category,
                categories,
                year: chrono::Utc::now().year(),
                metric_token: token,
                settings: settings.unwrap()
            }
            .into_response()
        }
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[cfg(test)]
mod tests {
    use crate::controllers;
    use crate::create_pool;
    use crate::CookieIdentityPolicy;
    use crate::IdentityService;
    use actix_web::http::header;
    use actix_web::{cookie::Cookie, http, http::StatusCode, test, web, App};
    use dotenv::dotenv;
    use std::str::FromStr;

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

    #[actix_rt::test]
    async fn test_article() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/blog").service(controllers::blog::show_article)),
        )
        .await;
        let res = test::TestRequest::get()
            .uri("/blog/articles/les-aventures-de-lulu-partie-3")
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_article_doesnt_exist() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/blog").service(controllers::blog::show_article)),
        )
        .await;
        let res = test::TestRequest::get()
            .uri("/blog/articles/les-aventures-de-lulu--3")
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_article_not_published() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/blog").service(controllers::blog::show_article)),
        )
        .await;
        let res = test::TestRequest::get()
            .uri("/blog/articles/les-aventures-de-lulu-partie-1")
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn test_article_not_seo() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/blog").service(controllers::blog::show_article)),
        )
        .await;
        let res = test::TestRequest::get()
            .uri("/blog/articles/les-aventures-de-lulu-partie-4")
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }
}
