use std::str::FromStr;

use crate::{services::metrics, utils::ua::UserAgent};
use actix_web::{http::HeaderValue, post, web, FromRequest, HttpRequest, HttpResponse};
use sqlx::PgPool;

pub async fn add(
    pool: &PgPool,
    req: &HttpRequest,
    belongs_to: metrics::BelongsTo,
) -> Result<Option<i32>, actix_web::Error> {
    if let Some(gar_log) = req.headers().get("GAR-Log") {
        if gar_log == HeaderValue::from_static("false") {
            return Ok(None);
        }
    }

    match UserAgent::from_request(req, &mut actix_web::dev::Payload::None).await {
        Ok(ua) => match metrics::add(
            &pool,
            belongs_to,
            // page_id,
            &req.peer_addr().unwrap().ip().to_string(),
            ua.name,
            ua.os,
            ua.category,
            match req.headers().get(actix_web::http::header::REFERER) {
                Some(referer) => match referer.to_str() {
                    Ok(referer) => Some(referer.to_string()),
                    _ => None,
                },
                _ => None,
            },
        )
        .await
        {
            Ok(id) => Ok(Some(id)),
            Err(err) => Err(actix_web::error::ErrorBadRequest(err)),
        },
        Err(e) => Err(e),
    }
}

#[derive(serde::Deserialize)]
pub struct Token {
    token: String,
}

#[post("/metrics/log")]
pub async fn log(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    form: web::Form<Token>,
) -> HttpResponse {
    match sqlx::types::Uuid::from_str(&form.token) {
        Ok(token) => {
            if metrics::tokens::exists(&pool, token).await {
                println!("Exist");

                if let Ok(id) = metrics::tokens::get_metric(&pool, token).await {
                    println!("ID {}", id);

                    match metrics::close(&pool, id).await {
                        Ok(_) => {
                            metrics::tokens::delete(&pool, token).await;
                            return HttpResponse::Ok().finish();
                        }
                        _ => return HttpResponse::InternalServerError().finish(),
                    }
                }
            }

            HttpResponse::NotFound().finish()
        }
        _ => HttpResponse::NotFound().finish(),
    }
}
