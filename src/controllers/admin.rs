use crate::services;
use actix_identity::Identity;
use actix_web::{get, web, Error, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

mod filters {
    pub fn rfc3339(date: &chrono::DateTime<chrono::Utc>) -> ::askama::Result<String> {
        Ok(date.to_rfc3339())
    }
}

#[get("")]
pub async fn index(session: Identity) -> Result<HttpResponse, Error> {
    if session.identity().is_some() {
        #[derive(Template)]
        #[template(path = "pages/admin/index.html")]
        struct Dashboard;

        return Dashboard {}.into_response();
    }

    #[derive(Template)]
    #[template(path = "pages/admin/login.html")]
    struct Login;

    Login {}.into_response()
}

#[get("/home")]
pub async fn home_page(id: Identity) -> Result<HttpResponse, Error> {
    if id.identity().is_some() {
        #[derive(Template)]
        #[template(path = "pages/admin/home.html")]
        struct Home;

        return Home {}.into_response();
    }

    #[derive(Template)]
    #[template(path = "pages/admin/home.html")]
    struct Login;

    Login {}.into_response()
}

#[get("/portfolio")]
pub async fn portfolio(session: Identity, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if session.identity().is_none() {
        return Ok(HttpResponse::Found().header("location", "/admin").finish());
    }

    #[derive(sqlx::FromRow, Serialize)]
    struct Project {
        id: i16,
        name: String,
        description: Option<String>,
        date: DateTime<Utc>,
    }

    let (categories, projects) = futures::join!(
        services::projects::categories::get_all(&pool, None),
        services::projects::get_all_spe::<Project>(&pool, "id, name, description, date", None)
    );

    match projects {
        Ok(projects) => {
            #[derive(Template)]
            #[template(path = "pages/admin/portfolio.html")]
            struct Portfolio {
                categories: Vec<services::projects::Category>,
                projects: Vec<Project>,
            }

            return Portfolio {
                categories,
                projects,
            }
            .into_response();
        }
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[get("/motion-design")]
pub async fn motion_design(
    session: Identity,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    if session.identity().is_none() {
        return Ok(HttpResponse::Found().header("location", "/admin").finish());
    }

    #[derive(Template)]
    #[template(path = "pages/admin/motion_design.html")]
    struct MotionDesign {
        link: String,
    }

    #[derive(sqlx::FromRow, Debug)]
    struct Chunk {
        content: serde_json::Value,
    }

    #[derive(serde::Deserialize)]
    struct ChunkData {
        link: String,
    }

    if let Ok(chunk) = services::pages::chunks::get::<Chunk>(&pool, "content", "link").await {
        if let Ok(data) = serde_json::from_value::<ChunkData>(chunk.content) {
            return MotionDesign { link: data.link }.into_response();
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/my_little_plus")]
pub async fn my_little_plus(
    pool: web::Data<PgPool>,
    session: Identity,
) -> Result<HttpResponse, Error> {
    if session.identity().is_none() {
        return Ok(HttpResponse::Found().header("location", "/admin").finish());
    }

    #[derive(sqlx::FromRow)]
    struct Chunk {
        content: serde_json::Value,
    }

    #[derive(Deserialize)]
    struct ChunkData {
        value: Option<String>,
    }

    let links = futures::try_join!(
        services::pages::chunks::get::<Chunk>(&pool, "content", "link_creations"),
        services::pages::chunks::get::<Chunk>(&pool, "content", "link_shootings")
    );

    if let Ok((link_creations, link_shootings)) = links {
        let creations = match serde_json::from_value::<ChunkData>(link_creations.content) {
            Ok(data) => data.value,
            Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
        };
        let shootings = match serde_json::from_value::<ChunkData>(link_shootings.content) {
            Ok(data) => data.value,
            Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
        };

        #[derive(Template)]
        #[template(path = "pages/admin/my_little_plus.html")]
        struct MyLittlePlus {
            creations: Option<String>,
            shootings: Option<String>,
        }

        return MyLittlePlus {
            creations,
            shootings,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/blog")]
async fn blog(session: Identity, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if session.identity().is_none() {
        return Ok(HttpResponse::Found().header("location", "/admin").finish());
    }

    #[derive(sqlx::FromRow, Serialize)]
    struct Category {
        id: i16,
        name: String,
    }

    #[derive(sqlx::FromRow, Serialize, Debug)]
    struct Article {
        id: i16,
        category_id: Option<i16>,
        title: String,
        description: Option<String>,
        date: DateTime<Utc>,
    }

    #[derive(Template)]
    #[template(path = "pages/admin/blog.html")]
    struct Blog {
        categories: Vec<Category>,
        articles: Vec<Article>,
    }

    let (categories, articles) = futures::join!(
        services::blog::categories::get_all::<Category>(&pool, "id, name", None, None),
        // TODO : refactor function to prevent test none
        services::blog::articles::get_all::<Article>(
            &pool,
            "ba.id, category_id, title, description, date",
            None,
            None,
            None
        )
    );

    Blog {
        categories,
        articles,
    }
    .into_response()
}

#[get("/parametres")]
pub async fn settings(session: Identity, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if session.identity().is_none() {
        return Ok(HttpResponse::Found().header("location", "/admin").finish());
    }

    match services::settings::get(&pool).await {
        Ok(settings) => {
            #[derive(Template)]
            #[template(path = "pages/admin/settings.html")]
            struct Setting {
                background_color: String,
                title_color: String,
                text_color: String
                // categories: Vec<services::projects::Category>,
                // projects: Vec<services::projects::Project>,
            }
        
            return Setting {
                background_color: settings.background_color,
                title_color: settings.title_color,
                text_color: settings.text_color
            }
            .into_response();
        }
        Err(e) => {
            eprintln!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::create_pool;
    use actix_web::{test, web, App};
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_index() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/admin").service(super::index)),
        )
        .await;
        let resp = test::TestRequest::get()
            .uri("/admin")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn text_index_with_cookie() {
        use actix_identity::{CookieIdentityPolicy, IdentityService};
        use std::str::FromStr;

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
                .service(web::scope("/admin").service(super::index)),
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
        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);
        let cookie = actix_web::http::Cookie::from_str(cookie.unwrap().to_str().unwrap()).unwrap();
        let res = test::TestRequest::get()
            .cookie(cookie)
            .uri("/admin")
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }
}
