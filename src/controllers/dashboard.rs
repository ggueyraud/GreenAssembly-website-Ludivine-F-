use actix_identity::Identity;
use actix_web::{get, web, Error, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use sqlx::PgPool;

use crate::services;

#[get("")]
pub async fn index(id: Identity) -> Result<HttpResponse, Error> {
    if let Some(id) = id.identity() {
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

#[get("/portfolio")]
pub async fn portfolio(id: Identity, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Some(id) = id.identity() {
        let (categories, projects) = futures::join!(
            services::projects::categories::get_all(&pool, None),
            services::projects::get_all(&pool, None)
        );

        #[derive(Template)]
        #[template(path = "pages/admin/portfolio.html")]
        struct Portfolio {
            categories: Vec<services::projects::Category>,
            projects: Vec<services::projects::Project>,
        }

        return Portfolio {
            categories,
            projects,
        }
        .into_response();
    }

    Ok(HttpResponse::Found().header("location", "/admin").finish())
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
