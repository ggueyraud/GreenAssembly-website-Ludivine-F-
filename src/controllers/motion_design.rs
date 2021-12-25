use actix_web::{HttpRequest, HttpResponse, web, get, put, Error};
use askama_actix::{Template, TemplateIntoResponse};
use sqlx::PgPool;
use crate::services;
use chrono::Datelike;
use actix_identity::Identity;

#[get("/motion-design")]
pub async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "motion-design").await {
        #[derive(sqlx::FromRow)]
        struct Chunk {
            content: serde_json::Value
        }

        #[derive(serde::Deserialize)]
        struct ChunkData {
            link: String
        }

        let link = if let Ok(chunk) = services::pages::chunks::get::<Chunk>(&pool, "content", "link").await {
            if let Ok(data) = serde_json::from_value::<ChunkData>(chunk.content) {
                data.link
            } else {
                return Ok(HttpResponse::InternalServerError().finish())
            }
        } else {
            return Ok(HttpResponse::InternalServerError().finish())
        };

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = super::metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        #[derive(Template)]
        #[template(path = "pages/motion_design.html")]
        struct MotionDesign {
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
            link: String
        }

        return MotionDesign {
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
            link
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[derive(serde::Deserialize)]
pub struct UpdateForm {
    link: String
}

#[put("")]
pub async fn update_informations(session: Identity, form: web::Json<UpdateForm>, pool: web::Data<PgPool>) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish()
    }

    match sqlx::query!(
        r#"UPDATE page_chunks
        SET content['link'] = $1
        WHERE identifier = 'link' AND page_id = 3"#,
        serde_json::Value::String(form.link.clone()),
    )
        .execute(pool.as_ref())
        .await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(_) => HttpResponse::InternalServerError().finish()
        }
}