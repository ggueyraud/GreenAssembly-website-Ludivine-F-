use super::metrics;
use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse, post, delete};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use sqlx::PgPool;
use actix_identity::Identity;
use serde::Deserialize;

#[derive(sqlx::FromRow)]
struct Article {
    title: String,
    uri: String,
    description: Option<String>,
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
                r#"ba.title,
                ba.uri,
                ba.description,
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
        #[template(path = "pages/blog/index.html")]
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
            r#"ba.title,
            ba.uri,
            ba.description,
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
    #[template(path = "pages/blog/category.html")]
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

#[get("/articles/{name}-{id}")]
async fn show_article(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    web::Path((name, id)): web::Path<(String, i16)>,
) -> Result<HttpResponse, Error> {
    if !services::blog::articles::exists_for_uri(&pool, &format!("{}-{}", name, id)).await {
        return Ok(HttpResponse::NotFound().finish());
    }

    #[derive(sqlx::FromRow, Debug)]
    struct Article {
        title: String,
        category_id: i16,
        cover_path: String,
        description: Option<String>,
        date: String,
        international_date: String,
        // As international date format
        modified_date: Option<String>,
        is_published: bool,
        is_seo: bool,
    }

    let article = services::blog::articles::get::<Article>(
        &pool,
        r#"title,
        category_id,
        f.path AS cover_path,
        description,
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
    .await;
    println!("{:?}", article);

    if let Ok(article) = article {
        if !article.is_published {
            return Ok(HttpResponse::NotFound().finish());
        }

        #[derive(sqlx::FromRow, Clone)]
        struct Block {
            title: Option<String>,
            content: String,
            left_column: bool,
            order: i16,
        }

        #[derive(Template)]
        #[template(path = "pages/blog/article.html")]
        struct BlogArticle {
            article: Article,
            category: Category,
            left_blocks: Vec<Block>,
            right_blocks: Vec<Block>,
            year: i32,
            metric_token: Option<String>,
        }

        #[derive(sqlx::FromRow)]
        struct Category {
            id: i16,
            name: String,
            uri: String,
        }

        let (metric_id, category, blocks) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::BlogPost(id)),
            services::blog::categories::get::<Category>(
                &pool,
                "id, name, uri",
                article.category_id
            ),
            services::blog::articles::blocks::get_all::<Block>(
                &pool,
                r#"title, content, left_column, "order""#,
                id
            )
        );

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = metric_id {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        return BlogArticle {
            article,
            category: category.unwrap(),
            left_blocks: blocks
                .iter()
                .filter(|&block| block.left_column == true)
                .cloned()
                .collect::<Vec<_>>(),
            right_blocks: blocks
                .iter()
                .filter(|&block| block.left_column == false)
                .cloned()
                .collect::<Vec<_>>(),
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[derive(Deserialize)]
pub struct NewCategoryForm {
    name: String,
    description: Option<String>,
    is_visible: Option<bool>,
    is_seo: Option<bool>
}

#[post("/categories")]
async fn insert_category(pool: web::Data<PgPool>, session: Identity, mut form: web::Form<NewCategoryForm>) -> HttpResponse {
    use slugmin::slugify;

    if let None = session.identity() {
        return HttpResponse::Unauthorized().finish()
    }

    form.name = form.name.trim().to_string();
    form.description = form.description.as_ref().and_then(|description| Some(description.trim().to_string()));

    if let Ok(id) = services::blog::categories::insert(
        &pool,
        &form.name,
        form.description.as_deref(),
        form.is_visible,
        form.is_seo
    ).await {
        if let Ok(_) = services::blog::categories::update_uri(&pool, id, &slugify(&format!("{}-{}", form.name, id))).await {
            return HttpResponse::Created().json(id)
        }
    }

    HttpResponse::InternalServerError().finish()
}

// TODO : update category

#[delete("/categories/{id}")]
async fn delete_category(pool: web::Data<PgPool>, session: Identity, web::Path(id): web::Path<i16>) -> HttpResponse {
    if let None = session.identity() {
        return HttpResponse::Unauthorized().finish()
    }

    if services::blog::categories::exists(&pool, id).await {
        services::blog::categories::delete(&pool, id).await;

        return HttpResponse::Ok().finish()
    }

    HttpResponse::NotFound().finish()
}

#[derive(Deserialize)]
pub struct ArticleBlock {
    title: Option<String>,
    content: Option<String>,
    left_column: bool,
    order: i16,
    pictures: Option<Vec<actix_extract_multipart::File>>
}

#[derive(Deserialize)]
pub struct NewArticleForm {
    cover: Option<actix_extract_multipart::File>,
    category_id: Option<i16>,
    title: String,
    description: Option<String>,
    is_published: bool,
    is_seo: bool,
    blocks: Vec<ArticleBlock>
}

#[post("/articles")]
async fn insert_article(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: actix_extract_multipart::Multipart<NewArticleForm>
) -> HttpResponse {
    use std::ops::DerefMut;

    if let None = session.identity() {
        return HttpResponse::Unauthorized().finish()
    }

    form.title = form.title.trim().to_string();
    form.description = form.description.as_ref().and_then(|description| Some(description.trim().to_string()));

    let mut transaction = pool.begin().await.unwrap();

    let cover_id: Option<i32> = None;
    // TODO : handle cover

    if let Ok(id) = services::blog::articles::insert(
        transaction.deref_mut(),
        // pool.get_ref(),
        form.category_id,
        cover_id,
        &form.title,
        form.description.as_deref(),
        form.is_published,
        form.is_seo
    ).await {
        for block in &form.blocks {
            if let Some(pictures) = &block.pictures {

            }

            services::blog::articles::blocks::insert(
                transaction.deref_mut(),
                id,
                block.title.as_deref(),
                block.content.as_deref(),
                block.left_column, block.order
            ).await;
            
        }

            transaction.commit().await.unwrap();

        return HttpResponse::Created().json(id)
    }

    HttpResponse::InternalServerError().finish()
}

#[delete("/articles/{id}")]
async fn delete_article(pool: web::Data<PgPool>, session: Identity, web::Path(id): web::Path<i16>) -> HttpResponse {
    if let None = session.identity() {
        return HttpResponse::Unauthorized().finish()
    }

    if services::blog::articles::exists(&pool, id).await {
        services::blog::articles::delete(&pool, id).await;

        return HttpResponse::Ok().finish()
    }

    HttpResponse::NotFound().finish()
}

#[cfg(test)]
mod tests {
    use crate::CookieIdentityPolicy;
    use crate::IdentityService;
    use crate::controllers;
    use std::str::FromStr;
    use crate::create_pool;
    use actix_web::{http::StatusCode, test, web, App, http, cookie::Cookie};
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

    #[actix_rt::test]
    async fn test_insert_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true)
                ))
                .data(pool.clone())
                .service(web::scope("/api/blog").service(controllers::blog::insert_category))
                .service(web::scope("/user").service(crate::controllers::user::login))
        )
        .await;

        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "hello@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        let cookie = res.headers().get(http::header::SET_COOKIE);

        assert!(cookie.is_some());
        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/api/blog/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Category 1",
                "is_visible": false,
                "is_seo": false
            }))
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_insert_category_not_logged() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/api/blog").service(controllers::blog::insert_category))
        )
        .await;

        let res = test::TestRequest::post()
            .uri("/api/blog/categories")
            .set_form(&serde_json::json!({
                "name": "Category 1",
                "is_visible": false,
                "is_seo": false
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn test_delete_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true)
                ))
                .data(pool.clone())
                .service(
                    web::scope("/api/blog")
                        .service(controllers::blog::insert_category)
                        .service(controllers::blog::delete_category)
                )
                .service(web::scope("/user").service(crate::controllers::user::login))
        )
        .await;

        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "hello@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        let cookie = res.headers().get(http::header::SET_COOKIE);

        assert!(cookie.is_some());
        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/api/blog/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Category 1",
                "is_visible": false,
                "is_seo": false
            }))
            .send_request(&mut app)
            .await;
        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::delete()
            .uri(&format!("/api/blog/categories/{}", id))
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_category_not_logged() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true)
                ))
                .data(pool.clone())
                .service(
                    web::scope("/api/blog")
                        .service(controllers::blog::insert_category)
                        .service(controllers::blog::delete_category)
                )
                .service(web::scope("/user").service(crate::controllers::user::login))
        )
        .await;

        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "hello@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        let cookie = res.headers().get(http::header::SET_COOKIE);

        assert!(cookie.is_some());
        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/api/blog/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Category 1",
                "is_visible": false,
                "is_seo": false
            }))
            .send_request(&mut app)
            .await;
        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::delete()
            .uri(&format!("/api/blog/categories/{}", id))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    }
}
