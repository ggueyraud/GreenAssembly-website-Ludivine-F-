use crate::{services::metrics, utils::ua::UserAgent};
use actix_web::{FromRequest, HttpRequest};
use sqlx::PgPool;

pub async fn add(pool: &PgPool, req: &HttpRequest, belongs_to: metrics::BelongsTo) -> bool {
    match UserAgent::from_request(req, &mut actix_web::dev::Payload::None).await {
        Ok(ua) => metrics::add(
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
        .is_ok(),
        _ => false,
    }
}
