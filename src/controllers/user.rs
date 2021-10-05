use crate::services;
use actix_identity::Identity;
use actix_web::{get, http, post, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use serde::Deserialize;
use sqlx::PgPool;

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

    let ip = if cfg!(debug_assertions) {
        "localhost".to_string()
    } else {
        req.peer_addr().unwrap().ip().to_string()
    };
    let attempts_counter = services::attempts::count(&pool, &ip, true).await;

    if attempts_counter > 10 {
        return HttpResponse::TooManyRequests().finish();
    }

    services::attempts::add(&pool, &form.email, &ip, true)
        .await
        .unwrap();

    let email_regex = Regex::new(r#"^(([^<>()\[\]\\.,;:\s@"]+(\.[^<>()\[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$"#).unwrap();

    if !email_regex.is_match(&form.email) {
        return HttpResponse::BadRequest().finish();
    }

    // TODO : move into model
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
                        services::attempts::clear(&pool, &ip).await;
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
            return HttpResponse::InternalServerError().finish();
        }
    }
}

#[derive(Deserialize)]
pub struct LostPasswordForm {
    email: String,
}

#[post("/lost-password")]
pub async fn lost_password(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mut form: web::Form<LostPasswordForm>,
) -> HttpResponse {
    use regex::Regex;

    form.email = form.email.trim().to_string();

    let ip = if cfg!(debug_assertions) {
        "localhost".to_string()
    } else {
        req.peer_addr().unwrap().ip().to_string()
    };
    let attempts_counter = services::attempts::count(&pool, &ip, false).await;

    if attempts_counter > 3 {
        return HttpResponse::TooManyRequests().finish();
    }

    services::attempts::add(&pool, &form.email, &ip, false)
        .await
        .unwrap();

    let email_regex = Regex::new(r#"^(([^<>()\[\]\\.,;:\s@"]+(\.[^<>()\[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$"#).unwrap();

    if !email_regex.is_match(&form.email) {
        return HttpResponse::BadRequest().finish();
    }

    if services::user::exist_for_email(&pool, &form.email).await {
        use lettre::{SmtpClient, Transport};
        use lettre_email::EmailBuilder;
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let mut token =
            String::from("@-_!ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");

        unsafe {
            token.as_mut_vec().shuffle(&mut rng);
            token = token[..60].to_string();
        }

        if attempts_counter >= 1 {
            services::attempts::clear(&pool, &ip).await;
        }

        sqlx::query!(
            r#"UPDATE "user" SET token = $1, token_validity_date = NOW() + interval '30 minutes'"#,
            token
        )
        .execute(pool.as_ref())
        .await
        .unwrap();

        let email = EmailBuilder::new()
            .to("contact@guillaume-gueyraud.fr")
            .from("hello@ludivinefarat.fr")
            .subject("Mot de passe oublié - Ludivine Farat")
            .html(r#"Vous avez effectué la demande de récupération de votre mot de passe, pour le récupérer merci de cliquer sur le bouton ci-dessous afin d'en saisir un nouveau.\n<a href="https://ludivinefarat.fr/admin/recuperation-mot-de-passe">Récupérer mon mot de passe</a>"#)
            .build();

        if let Ok(email) = email {
            let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();

            if let Ok(_) = mailer.send(email.into()) {
                return HttpResponse::Ok().json(serde_json::json!({
                    "valid": true
                }));
            }
        }

        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(serde_json::json!({
        "valid": false
    }))
}

#[derive(Deserialize)]
pub struct PasswordRecoveryForm {
    password: String,
    token: String,
}

// TODO : need to be finish
#[post("/password-recovery")]
pub async fn password_recovery(
    pool: web::Data<PgPool>,
    mut form: web::Form<PasswordRecoveryForm>,
) -> HttpResponse {
    use regex::Regex;

    form.password = form.password.trim().to_owned();

    println!("{} - {}", form.password, form.password.len());

    //^(?:.*[a-z]).{8,}$
    // ^(?:.*\d)(?:.*[a-z])(?:.*[A-Z])(?:.*[!@#$%^&*]).{8,}$
    let password_regex = Regex::new(r#"^.{7,}$"#).unwrap();

    if password_regex.is_match(&form.password) {
        return HttpResponse::Ok().finish();
    }

    HttpResponse::BadRequest().finish()
}

#[get("/logout")]
pub async fn logout(id: Identity) -> HttpResponse {
    id.forget();

    HttpResponse::Found().header("location", "/admin").finish()
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
