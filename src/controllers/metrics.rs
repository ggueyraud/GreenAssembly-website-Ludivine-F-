use std::str::FromStr;

use crate::{services::metrics, utils::ua::UserAgent};
use actix_web::{http::HeaderValue, post, web, get, FromRequest, HttpRequest, HttpResponse};
use sqlx::PgPool;
use serde::{Serialize, Deserialize};
use ring::digest;
use chrono::{DateTime, Utc};

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

#[derive(Deserialize)]
pub struct PageInformations {
    path: String,
    sid: String,
}

#[get("/metrics/token")]
pub async fn create(pool: web::Data<PgPool>, req: HttpRequest, infos: web::Query<PageInformations>) -> HttpResponse {
    // if let Ok(page_id) = pages::get_id(&pool, &infos.path).await {
    //     if let Ok(ua) = UserAgent::from_request(&req, &mut actix_web::dev::Payload::None).await {
    //         let sid = match Uuid::from_str(infos.sid.as_str()) {
    //             Ok(val) => val,
    //             Err(_) =>  return HttpResponse::BadRequest().finish()
    //         };

    //         let digest_ip = digest::digest(&digest::SHA256, req.peer_addr().unwrap().ip().to_string().as_bytes());
    //         let digest_ip = format!("{:?}", digest_ip);

    //         if let Ok(id) = metrics::add(
    //             &pool,
    //             Some(sid),
    //             page_id,
    //             &digest_ip,
    //             ua.product.name.clone(),
    //             ua.os.name.clone(),
    //             ua.device.name.clone(),
    //             match req.headers().get(actix_web::http::header::REFERER) {
    //                 Some(referer) => match referer.to_str() {
    //                     Ok(referer) => Some(referer.to_string()),
    //                     _ => None,
    //                 },
    //                 _ => None,
    //             },
    //         )
    //         .await {
    //             return HttpResponse::Ok().body(id.to_hyphenated().to_string())
    //         }
    //     }
    // }

    HttpResponse::InternalServerError().finish()
}

#[derive(Deserialize)]
pub struct Token {
    #[serde(rename(deserialize = "sid"))]
    session_id: Option<String>,
    token: String
}

#[post("/metrics/log")]
pub async fn log(
    pool: web::Data<PgPool>,
    form: web::Form<Token>
) -> HttpResponse {
    // if let Ok(token) = sqlx::types::Uuid::from_str(&form.token) {
    //     if metrics::exists(&pool, token).await {
    //         let session_id: Option<Uuid> = match &form.session_id {
    //             Some(val) => match Uuid::from_str(val.as_str()) {
    //                     Ok(res) => Some(res),
    //                     Err(_) => None
    //                 }
    //             None => None
    //         };

    //         if let Ok(true) = metrics::update_end_date(&pool, session_id, token).await {
    //             return HttpResponse::Ok().finish()
    //         }
    //     }
    // }

    HttpResponse::NotFound().finish()
}

// ------------------------------------------------------------------------------ //
// -------------------------------- USER SESSION -------------------------------- //
// ------------------------------------------------------------------------------ //

#[derive(Serialize)]
pub struct SessionData {
    pub sid: String,
    pub vud: DateTime<Utc>
}

#[get("/metrics/session")]
pub async fn create_session(pool: web::Data<PgPool>, req: HttpRequest) -> HttpResponse {
    // let digest_ip = digest::digest(&digest::SHA256, req.peer_addr().unwrap().ip().to_string().as_bytes());
    // let digest_ip = format!("{:?}", digest_ip);
    
    // if let Ok(session_data) = metrics::sessions::add(
    //     &pool,
    //     &digest_ip
    // )
    // .await {
    //     let sid = session_data.0.to_hyphenated().to_string();
    //     let vud = session_data.1;
    //     return HttpResponse::Ok().json(SessionData {
    //         sid,
    //         vud
    //     })
    // }

    HttpResponse::InternalServerError().finish()
}