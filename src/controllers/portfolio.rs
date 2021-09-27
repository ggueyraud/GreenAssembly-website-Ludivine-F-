use super::metrics;
use crate::services;
use actix_extract_multipart::*;
use actix_identity::Identity;
use actix_web::{delete, get, patch, post, put, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::PgPool;
use std::{collections::HashSet, path::Path};

#[get("")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "portfolio").await {
        use slugmin::slugify;

        let mut token: Option<String> = None;
        if let Ok(Some(id)) =
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await
        {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        #[derive(Debug)]
        struct Illustration {
            path: String,
            name: Option<String>,
        }

        #[derive(Template)]
        #[template(path = "components/project_tile.html")]
        struct ProjectTile {
            name: String,
            uri: String,
            illustration: Illustration,
            categories: Vec<services::projects::Category>,
        }

        #[derive(Template)]
        #[template(path = "pages/portfolio.html")]
        struct Portfolio {
            categories: Vec<services::projects::Category>,
            projects: Vec<ProjectTile>,
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
        }

        let (_, projects, categories) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::projects::get_all(&pool, None),
            services::projects::categories::get_all(&pool)
        );
        let mut formatted_projects = vec![];

        for project in &projects {
            let project_categories = sqlx::query!(
                "SELECT category_id FROM projects_categories WHERE project_id = $1",
                project.id
            )
            .fetch_all(pool.as_ref())
            .await
            .unwrap();
            let mut c = vec![];

            for project_category in project_categories {
                if let Some(category) = categories
                    .iter()
                    .find(|category| category.id == project_category.category_id)
                {
                    c.push(category.clone());
                }
            }

            formatted_projects.push(ProjectTile {
                name: project.name.clone(),
                uri: slugify(&format!("{}-{}", project.name, project.id)),
                illustration: sqlx::query_as!(
                    Illustration,
                    r#"SELECT
                        f.path AS "path", f.name AS "name"
                    FROM project_assets pa
                    JOIN files f ON f.id = pa.file_id
                    WHERE pa.project_id = $1 AND pa.order = 1"#,
                    project.id
                )
                .fetch_one(pool.as_ref())
                .await
                .unwrap(),
                categories: c,
            });
        }

        return Portfolio {
            categories,
            projects: formatted_projects,
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/{name}-{id}")]
async fn get_project(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    web::Path((_, id)): web::Path<(String, i16)>,
) -> Result<HttpResponse, Error> {
    if services::projects::exists(&pool, id).await {
        let (project, assets) = futures::join!(
            services::projects::get(&pool, id),
            services::projects::assets::get_all(&pool, id)
        );

        if let Ok(project) = services::projects::get(&pool, id).await {
            let mut token: Option<String> = None;
            let assets = services::projects::assets::get_all(&pool, id).await;

            if let Ok(Some(id)) =
                metrics::add(&pool, &req, services::metrics::BelongsTo::Project(id)).await
            {
                if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                    token = Some(metric_token.to_string());
                }
            }

            #[derive(Template)]
            #[template(path = "pages/portfolio_project.html")]
            struct PortfolioProject<'a> {
                title: String,
                description: Option<String>,
                content: String,
                date: String,
                international_date: String,
                asset_0: Option<&'a services::projects::Asset>,
                asset_1: Option<&'a services::projects::Asset>,
                assets: Option<Vec<services::projects::Asset>>,
                year: i32,
                metric_token: Option<String>,
            }

            return PortfolioProject {
                title: project.name,
                description: project.description,
                content: project.content,
                date: project.date.format("%d/%m/%Y").to_string(),
                international_date: project.date.to_rfc3339(),
                asset_0: assets.get(0),
                asset_1: assets.get(1),
                assets: if assets.len() - 2 > 0 {
                    Some(assets.get(2..).unwrap().to_vec())
                } else {
                    None
                },
                year: chrono::Utc::now().year(),
                metric_token: token,
            }
            .into_response();
        }
    }

    Ok(HttpResponse::NotFound().finish())
}

#[derive(Deserialize)]
pub struct CategoryForm {
    name: String,
    order: i16,
}

impl CategoryForm {
    fn is_valid(&mut self) -> bool {
        self.name = self.name.trim().to_string();

        self.name.len() >= 2 && self.name.len() <= 120
    }
}

#[post("/categories")]
async fn create_category(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: web::Form<CategoryForm>,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if !form.is_valid() {
            return HttpResponse::BadRequest().finish();
        }

        return match services::projects::categories::insert(&pool, &form.name, form.order).await {
            Ok(id) => HttpResponse::Ok().json(id),
            _ => HttpResponse::InternalServerError().finish(),
        };
    }

    HttpResponse::Unauthorized().finish()
}

#[put("/categories/{id}")]
async fn update_category(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: web::Form<CategoryForm>,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if !form.is_valid() {
            return HttpResponse::BadRequest().finish();
        }

        return match services::projects::categories::update(&pool, id, &form.name, form.order).await
        {
            Ok(_) => HttpResponse::Ok().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        };
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/categories/{id}")]
async fn delete_category(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if services::projects::categories::exists(&pool, id).await {
            services::projects::categories::delete(&pool, id).await;

            return HttpResponse::Ok().finish();
        }

        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Unauthorized().finish()
}

impl services::projects::ProjectInformations {
    async fn is_valid(&mut self, pool: &PgPool) -> bool {
        use ammonia::Builder;

        self.name = self.name.trim().to_string();
        if let Some(description) = &mut self.description {
            *description = description.trim().to_string();
        }
        // Sanitize content for only print allowed html tags
        let mut allowed_tags: HashSet<&str> = HashSet::new();
        allowed_tags.insert("b");
        allowed_tags.insert("h2");
        allowed_tags.insert("h3");
        allowed_tags.insert("ul");
        allowed_tags.insert("li");
        allowed_tags.insert("a");
        self.content = Builder::default()
            .tags(allowed_tags)
            .clean(self.content.trim())
            .to_string();

        self.name.len() >= 2 && self.name.len() <= 120 && self.content.len() >= 30
        // && self.category_id > 0
        // && services::projects::categories::exists(pool, self.category_id).await
    }
}

#[post("/projects")]
pub async fn insert_project(
    pool: web::Data<PgPool>,
    mut form: web::Form<services::projects::ProjectInformations>,
    session: Identity,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if !form.is_valid(&pool).await {
            return HttpResponse::BadRequest().finish();
        }

        return match services::projects::insert(&pool, &*form).await {
            Ok(id) => HttpResponse::Ok().json(id),
            _ => HttpResponse::InternalServerError().finish(),
        };
    }

    HttpResponse::Unauthorized().finish()
}

#[patch("/projects/{id}")]
pub async fn update_project(
    pool: web::Data<PgPool>,
    mut form: web::Form<services::projects::ProjectInformations>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if !form.is_valid(&pool).await {
            return HttpResponse::BadRequest().finish();
        }

        return match services::projects::update(&pool, id, &*form).await {
            Ok(id) => HttpResponse::Ok().json(id),
            _ => HttpResponse::InternalServerError().finish(),
        };
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/projects/{id}")]
async fn delete_project(
    pool: web::Data<PgPool>,
    web::Path(id): web::Path<i16>,
    session: Identity,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if services::projects::exists(&pool, id).await {
            let assets = services::projects::assets::get_all(&pool, id).await;

            assets.iter().for_each(|asset| {
                // TODO : remove all differents file formats

                let path = format!("./uploads/{}", asset.path);
                let path = Path::new(&path);

                if path.exists() {
                    std::fs::remove_file(path).unwrap();
                }
            });

            services::projects::delete(&pool, id).await;

            return HttpResponse::Ok().finish();
        }

        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Unauthorized().finish()
}

#[derive(Deserialize)]
pub struct AssetForm {
    // TODO : remplace with video
    file: File,
    name: Option<String>,
    order: i16,
    is_visible: bool,
}

impl AssetForm {
    fn sanitize(&mut self) {
        if let Some(name) = &mut self.name {
            *name = name.trim().to_string();
        }
    }

    fn is_valid(&mut self) -> bool {
        self.sanitize();

        self.order > 0
    }
}

#[post("/projects/{id}/assets")]
pub async fn insert_asset(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
    payload: actix_multipart::Multipart,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        let mut form = match extract_multipart::<AssetForm>(payload).await {
            Ok(data) => data,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };
        if !form.is_valid() {
            return HttpResponse::BadRequest().finish();
        }

        if services::projects::exists(&pool, id).await {
            use std::fs::File;
            use std::io::prelude::*;
            use webp::Encoder;

            let image = image::load_from_memory(&form.file.data()).unwrap();

            let name = if let Some(name) = form.name {
                use slugmin::slugify;

                slugify(&format!("{}_{}", name, id))
            } else {
                format!("{}_{}", id, chrono::Utc::now().timestamp())
            };

            match form.file.file_type() {
                FileType::ImageJPEG
                | FileType::ImagePNG
                | FileType::ImageWEBP
                | FileType::ImageGIF => {
                    let format = if image.color().has_alpha() {
                        image::ImageFormat::Png
                    } else {
                        image::ImageFormat::Jpeg
                    };

                    match format {
                        image::ImageFormat::Png => {}
                        _ => {}
                    }

                    if image.color().has_alpha() {
                        image
                            .thumbnail(500, 500)
                            .save_with_format(
                                format!("./uploads/mobile/{}.png", name),
                                image::ImageFormat::Jpeg,
                            )
                            .unwrap();

                        image
                            .resize(800, 800, image::imageops::CatmullRom)
                            .save_with_format(
                                format!("./uploads/{}.png", name),
                                image::ImageFormat::Jpeg,
                            )
                            .unwrap();
                    } else {
                        image
                            .thumbnail(500, 500)
                            .save_with_format(
                                format!("./uploads/mobile/{}.jpg", name),
                                image::ImageFormat::Jpeg,
                            )
                            .unwrap();
                    }

                    let image_webp =
                        Encoder::from_image(&image.resize(500, 500, image::imageops::CatmullRom))
                            .encode(100.0);
                    let v = image_webp.iter().map(|a| *a).collect::<Vec<u8>>();
                    let mut file = File::create(format!("./uploads/mobile/{}.webp", name)).unwrap();
                    file.write_all(&v).unwrap();

                    // Desktop image size
                    let image_webp =
                        Encoder::from_image(&image.resize(800, 800, image::imageops::CatmullRom))
                            .encode(100.0);
                    let v = image_webp.iter().map(|a| *a).collect::<Vec<u8>>();
                    let mut file = File::create(format!("./uploads/{}.webp", name)).unwrap();
                    file.write_all(&v).unwrap();
                }
                // MP4
                _ => {}
            }

            // services::projects::assets::insert(&pool, id, path, form.order, form.is_visible).await;

            return HttpResponse::Created().finish();
        }

        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Unauthorized().finish()
}

#[derive(Deserialize)]
pub struct UpdateAssetForm {
    name: Option<String>,
    order: i16,
}

#[patch("/projects/{project_id}/assets/{asset_id}")]
async fn update_asset(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path((project_id, asset_id)): web::Path<(i16, i16)>,
    form: web::Form<UpdateAssetForm>,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if services::projects::assets::exists(&pool, project_id, asset_id).await {
            match sqlx::query!(
                "CALL update_asset($1, $2, $3)",
                asset_id,
                form.order,
                form.name
            )
            .execute(pool.as_ref())
            .await
            {
                Ok(_) => return HttpResponse::Ok().finish(),
                _ => return HttpResponse::InternalServerError().finish(),
            }
        }

        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/projects/{project_id}/assets/{asset_id}")]
async fn delete_asset(
    pool: web::Data<PgPool>,
    web::Path((project_id, asset_id)): web::Path<(i16, i16)>,
    session: Identity,
) -> HttpResponse {
    if let Some(_) = session.identity() {
        if services::projects::assets::exists(&pool, project_id, asset_id).await {
            if let Ok(asset) = services::projects::assets::get(&pool, asset_id).await {
                // TODO : remove all differents file formats

                let path = format!("./uploads/{}", asset.path);
                let path = Path::new(&path);

                if path.exists() {
                    std::fs::remove_file(path).unwrap();
                }

                return HttpResponse::Ok().finish();
            }
        }

        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Unauthorized().finish()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::create_pool;
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{cookie::Cookie, test, web, App};
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_index() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/portfolio").service(super::index)),
        )
        .await;
        let resp = test::TestRequest::get()
            .uri("/portfolio")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_project() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/portfolio").service(super::get_project)),
        )
        .await;
        let resp = test::TestRequest::get()
            .uri("/portfolio/lorem-1")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_create_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(web::scope("/portfolio").service(super::create_category)),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 1
            }))
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_create_category_not_logged() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(web::scope("/portfolio").service(super::create_category)),
        )
        .await;

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 1
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 401);
    }

    #[actix_rt::test]
    async fn test_update_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::update_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 2
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::put()
            .uri(&format!("/portfolio/categories/{}", id))
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Dolor sit amet",
                "order": 2
            }))
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_update_category_not_logged() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::update_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 40
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::put()
            .uri(&format!("/portfolio/categories/{}", id))
            .set_form(&serde_json::json!({
                "name": "Dolor sit amet",
                "order": 50
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 401);
    }

    #[actix_rt::test]
    async fn test_update_category_doesnt_exist() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::update_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());
        assert!(res.status().is_success());

        let res = test::TestRequest::put()
            .uri("/portfolio/categories/999")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Dolor sit amet",
                "order": 2
            }))
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
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
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::delete_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());
        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 3
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::delete()
            .uri(&format!("/portfolio/categories/{}", id))
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
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::delete_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 10
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::delete()
            .uri(&format!("/portfolio/categories/{}", id))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 401);
    }

    #[actix_rt::test]
    async fn test_delete_category_doesnt_exist() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::delete_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        let res = test::TestRequest::delete()
            .uri("/portfolio/categories/9999")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 404);
    }

    // TODO : implement delete projects tests
    #[actix_rt::test]
    async fn test_delete_project() {
        //     dotenv().ok();

        //     let pool = create_pool().await.unwrap();
        //     let mut app =
        //         test::init_service(App::new().data(pool.clone()).service(super::get_project)).await;
        //     let resp = test::TestRequest::get()
        //         .uri("/portfolio/lorem-1")
        //         .send_request(&mut app)
        //         .await;

        //     assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_project_doesnt_exists() {}

    #[actix_rt::test]
    async fn test_delete_project_not_logged() {}
}
