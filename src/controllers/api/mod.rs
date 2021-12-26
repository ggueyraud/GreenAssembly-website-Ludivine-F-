use crate::{services, utils::{image::Uploader, patch::Patch}};
use actix_extract_multipart::{File, Multipart};
use actix_identity::Identity;
use actix_web::{patch, post, put, web, HttpResponse};
use regex::Regex;
use serde::Deserialize;
use sqlx::PgPool;

pub mod blog;
pub mod portfolio;

#[derive(Deserialize)]
pub struct UpdateForm {
    link: String,
}

#[put("")]
pub async fn update_motion_design_informations(
    session: Identity,
    form: web::Json<UpdateForm>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    match sqlx::query!(
        r#"UPDATE page_chunks
        SET content['link'] = $1
        WHERE identifier = 'link' AND page_id = 3"#,
        serde_json::Value::String(form.link.clone()),
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize)]
pub struct HomeImage {
    pub image: File,
}

#[patch("/image")]
pub async fn update_home_informations(
    session: Identity,
    data: Multipart<HomeImage>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let mut uploader = crate::utils::image::Uploader::new();

    if !&["image/jpeg", "image/png", "image/webp"].contains(&data.image.file_type().as_str()) {
        return HttpResponse::BadRequest().finish();
    }

    if let Ok(image) = image::load_from_memory(data.image.data()) {
        if uploader
            .handle(&image, "index", None, Some((1000, 1000)))
            .is_err()
        {
            return HttpResponse::BadRequest().finish();
        }

        uploader.clear();

        return HttpResponse::Ok().finish();
    }

    HttpResponse::InternalServerError().finish()
}

#[derive(Deserialize)]
pub struct FormUpdateLittlePlus {
    pub creations: Option<String>,
    pub shootings: Option<String>,
}

#[patch("/links")]
async fn update_little_plus_informations(
    pool: web::Data<PgPool>,
    session: Identity,
    links: web::Json<FormUpdateLittlePlus>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let http_regex = Regex::new(r"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$").unwrap();
    let mut fut = vec![];

    if let Some(url) = &links.creations {
        if http_regex.is_match(url) {
            fut.push(
                sqlx::query!(
                    r#"UPDATE page_chunks
                    SET content['value'] = $1
                    WHERE identifier = 'link_creations' AND page_id = 4"#,
                    serde_json::Value::String((*url).clone())
                )
                .execute(pool.as_ref()),
            );
        } else {
            return HttpResponse::BadRequest().finish();
        }
    }
    if let Some(url) = &links.shootings {
        if http_regex.is_match(url) {
            fut.push(
                sqlx::query!(
                    r#"UPDATE page_chunks
                    SET content['value'] = $1
                    WHERE identifier = 'link_shootings' AND page_id = 4"#,
                    serde_json::Value::String((*url).clone())
                )
                .execute(pool.as_ref()),
            );
        } else {
            return HttpResponse::BadRequest().finish();
        }
    }

    match futures::future::try_join_all(fut).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }

    // match crate::services::my_little_plus::edit_links(&pool, &links).await {
    //     Ok(_) => HttpResponse::Ok().finish(),
    //     Err(_) => HttpResponse::InternalServerError().finish()
    // }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct UpdateParametersForm {
    #[serde(skip_serializing)]
    logo: Patch<File>,
    #[serde(skip_serializing)]
    favicon: Patch<File>,
    #[serde(default)]
    background_color: Patch<String>,
    #[serde(default)]
    title_color: Patch<String>,
    #[serde(default)]
    text_color: Patch<String>,
}

#[patch("")]
pub async fn update_settings(
    session: Identity,
    pool: web::Data<PgPool>,
    form: Multipart<UpdateParametersForm>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let mut uploader = Uploader::new();

    if let Patch::Value(logo) = &form.logo {
        match image::load_from_memory(logo.data()) {
            Ok(image) => {
                // if uploader
                //     .handle(&image, "logo", Some(()))
                //     .is_err() {
                //         return HttpResponse::BadRequest().finish()
                //     }
            }
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }

    if let Patch::Value(favicon) = &form.favicon {
        match image::load_from_memory(favicon.data()) {
            Ok(favicon) => {
                // if uploader
                //     .handle(&favicon, "favicon", Some((64, 64)), Some(()))
                //     .is_err() {
                //         return HttpResponse::BadRequest().finish()
                //     }
            }
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }

    let mut fields_need_update = crate::utils::patch::extract_fields(&*form);

    println!("Form : {:?} | Fields need udpate : {:?}", *form, fields_need_update);

    match services::settings::partial_update(pool.as_ref(), fields_need_update).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ContactForm {
    firstname: String,
    lastname: String,
    phone_number: Option<String>,
    email: String,
    content: String,
}

#[post("/contact")]
pub async fn contact(mut form: web::Form<ContactForm>) -> HttpResponse {
    use lettre::{SmtpClient, Transport};
    use lettre_email::EmailBuilder;

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
        if let Ok(stmt_client) = SmtpClient::new_unencrypted_localhost() {
            if stmt_client.transport().send(email.into()).is_ok() {
                return HttpResponse::Ok().finish();
            }
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
