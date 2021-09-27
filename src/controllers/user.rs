use crate::services;
use actix_identity::Identity;
use actix_web::{Error, HttpRequest, HttpResponse, get, http, post, web};
use askama_actix::{Template, TemplateIntoResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[get("/connexion")]
pub async fn show_login(id: Identity) -> Result<HttpResponse, Error> {
    if id.identity().is_some() {
        return Ok(HttpResponse::Ok().header(http::header::LOCATION, "/"))
    }

    #[derive(Template)]
    #[template(path = "pages/login.html")]
    struct Login {
        title: String
    }

    return Login {
        title: String::from("Connexion")
    }.into_response()
}

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}
#[post("/login")]
pub async fn login(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mut form: web::Form<LoginForm>,
    id: Identity,
) -> HttpResponse {
    use regex::Regex;

    form.email = form.email.trim().to_string();
    form.password = form.password.trim().to_string();

    let email_regex = Regex::new(r#"^(([^<>()\[\]\\.,;:\s@"]+(\.[^<>()\[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$"#).unwrap();

    if !email_regex.is_match(&form.email) {
        return HttpResponse::BadRequest().finish();
    }

    let ip = if cfg!(debug_assertions) {
        "localhost".to_string()
    } else {
        req.peer_addr().unwrap().ip().to_string()
    };

    // let ip = ;
    let attempts_counter = services::login_attempts::count(&pool, &ip).await;

    if attempts_counter > 10 {
        return HttpResponse::TooManyRequests().finish();
    }

    match sqlx::query!(
        r#"SELECT password FROM "user" WHERE email = $1 LIMIT 1"#,
        form.email
    )
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(row) => {
            use argon2::{
                password_hash::{PasswordHash, PasswordVerifier},
                Argon2,
            };

            let argon2 = Argon2::default();

            let parsed_hash = PasswordHash::new(&row.password).unwrap();

            match argon2.verify_password(form.password.as_bytes(), &parsed_hash) {
                Ok(_) => {
                    if attempts_counter >= 1 {
                        services::login_attempts::clear(&pool, &ip).await;
                    }

                    // TODO : see what to save in session
                    // id.remember(serde_json::to_string(&user).unwrap());
                    id.remember(1.to_string());

                    return HttpResponse::Ok().json(serde_json::json!({
                        "valid": true
                    }));
                }
                _ => {
                    if attempts_counter + 1 >= 10 {
                        // TODO : inform fail2ban to ban this IP
                    }

                    return HttpResponse::Ok().json(serde_json::json!({
                        "valid": false
                    }));
                }
            }
        }
        _ => {
            use crate::utils::ua::UserAgent;
            use actix_web::{dev::Payload, FromRequest};

            match UserAgent::from_request(&req, &mut Payload::None).await {
                Ok(ua) => {
                    services::login_attempts::add(
                        &pool,
                        &form.email,
                        &ip,
                        ua.name.as_deref(),
                        ua.os.as_deref(),
                        ua.category.as_deref(),
                    )
                    .await
                    .unwrap();
                }
                _ => (),
            }

            return HttpResponse::BadRequest().finish();
        }
    }
}

#[get("/logout")]
pub async fn logout(id: Identity) -> HttpResponse {
    id.forget();

    HttpResponse::Found().header("location", "/").finish()
}

#[cfg(test)]
mod tests {
    use crate::create_pool;
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{test, web, App};
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_login() {
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
                .service(web::scope("/user").service(super::login)),
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
    }

    #[actix_rt::test]
    async fn test_invalid_login() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/user").service(super::login)),
        )
        .await;
        let resp = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contat@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }
}
