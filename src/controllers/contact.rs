use super::metrics;
use crate::services;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::PgPool;

#[get("/contact")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "contact").await {
        let mut token: Option<String> = None;

        if let Ok(Some(id)) =
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await
        {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        println!("Token {:?}", token);

        #[derive(Template)]
        #[template(path = "contact.html")]
        struct Contact {
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
        }

        return Contact {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[derive(Deserialize, Debug)]
pub struct ContactForm {
    firstname: String,
    lastname: String,
    phone_number: Option<String>,
    email: String,
    content: String,
}

#[post("/contact")]
async fn post(mut form: web::Form<ContactForm>) -> HttpResponse {
    use lettre::{SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    use regex::Regex;

    // Trim form fields
    form.firstname = form.firstname.trim().to_string();
    form.lastname = form.lastname.trim().to_string();
    if let Some(phone_number) = &form.phone_number {
        form.phone_number = Some(phone_number.trim().to_string());
    }
    form.email = form.email.trim().to_string();
    form.content = form.content.trim().to_string();

    // Controls form informations
    let phone_regex = Regex::new(r"^((\+)33|0|0033)[1-9](\d{2}){4}$").unwrap();
    let email_regex = Regex::new(r#"^(([^<>()\[\]\\.,;:\s@"]+(\.[^<>()\[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$"#).unwrap();

    if form.firstname.len() < 2
        || form.firstname.len() > 120
        || form.lastname.len() < 2
        || form.lastname.len() > 120
        || (form.phone_number.is_some()
            && !phone_regex.is_match(form.phone_number.as_ref().unwrap()))
        || !email_regex.is_match(&form.email)
        || form.content.len() < 30
        || form.content.len() > 500
    {
        return HttpResponse::BadRequest().finish();
    }

    // Format email content
    let mut content = format!(
        "<u>Nom :</u> {}<br />
        <u>Prénom :</u> {} <br />",
        form.lastname, form.firstname
    );

    if let Some(phone_number) = &form.phone_number {
        content += &format!("<u>Numéro :</u> {}<br />", phone_number);
    }

    content += &format!("<u>Message :</u> {}<br />", form.content);

    // TODO : see to pass as Secure (SSL or equivalent)
    let email = EmailBuilder::new()
        .to("contact@ludivinefarat.fr")
        .from(form.email.as_str())
        .subject("")
        .html(content.as_str())
        .build();

    if let Ok(email) = email {
        let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();

        if let Ok(_) = mailer.send(email.into()) {
            return HttpResponse::Ok().finish();
        }
    }

    HttpResponse::InternalServerError().finish()
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
            .uri("/contact")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_form_valid_data() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(super::post)).await;
        let resp = test::TestRequest::post()
            .uri("/contact")
            .set_form(&serde_json::json!({
                "firstname": "Guillaume",
                "lastname": "Gueyraud",
                "phone_number": "0767656195",
                "email": "contact@guillaume-gueyraud.fr",
                "content": "Lorem ipsum dolor sit egestas."
            }))
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_form_invalid_phone_number() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(super::post)).await;
        let resp = test::TestRequest::post()
            .uri("/contact")
            .set_form(&serde_json::json!({
                "firstname": "Guillaume",
                "lastname": "Gueyraud",
                "phone_number": "x0767656195",
                "email": "contact@guillaume-gueyraud.fr",
                "content": "Lorem ipsum dolor sit egestas."
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_form_invalid_email() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(super::post)).await;
        let resp = test::TestRequest::post()
            .uri("/contact")
            .set_form(&serde_json::json!({
                "firstname": "Guillaume",
                "lastname": "Gueyraud",
                "phone_number": "+33767656195",
                "email": "cont;act@guillaume-gueyraud.fr",
                "content": "Lorem ipsum dolor sit egestas."
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(resp.status(), 400);
    }
}
