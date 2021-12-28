use std::str::FromStr;

use crate::{services, utils::ua::UserAgent};
use actix_web::{http::HeaderValue, post, web, get, FromRequest, HttpRequest, HttpResponse};
use sqlx::{PgPool, types::Uuid, FromRow};
use serde::{Serialize, Deserialize};
use ring::digest;
use chrono::{DateTime, Utc};

pub async fn add(
    pool: &PgPool,
    req: &HttpRequest,
    belongs_to: services::metrics::BelongsTo,
) -> Result<Option<Uuid>, actix_web::Error> {
    if let Some(gar_log) = req.headers().get("GAR-LOG") {
        if gar_log == HeaderValue::from_static("false") {
            return Ok(None)
        }
    }

    let ua = UserAgent::from_request(req, &mut actix_web::dev::Payload::None).await?;
    let digest_ip = digest::digest(&digest::SHA256, req.peer_addr().unwrap().ip().to_string().as_bytes());
    let digest_ip = format!("{:?}", digest_ip);

    match services::metrics::add(
        &pool,
        belongs_to,
        None,
        &digest_ip,
        ua.name.clone(),
        ua.os.clone(),
        ua.category.clone(),
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
        Err(e) => Err(actix_web::error::ErrorBadRequest(e))
    }
}

#[derive(Deserialize)]
pub enum BelongsTo {
    Page,
    Project,
    BlogPost,
}

#[derive(Deserialize)]
pub struct PageInformations {
    path: String,
    sid: String,
    belongs_to: BelongsTo
}

#[get("/metrics/token")]
pub async fn create(pool: web::Data<PgPool>, req: HttpRequest, infos: web::Query<PageInformations>) -> HttpResponse {
    let mut id: Option<i16> = None;
    match infos.belongs_to {
        BelongsTo::Page => {
            #[derive(FromRow)]
            struct Page {
                id: i16
            }

            match services::pages::get::<Page>(&pool, "id", &infos.path).await {
                Ok(page) => id = Some(page.id),
                Err(e) => {
                    println!("{:?}", e);
                    return HttpResponse::InternalServerError().finish()
                }
            }
        },
        BelongsTo::BlogPost | BelongsTo::Project => {
            match infos.path.split('-').collect::<Vec<_>>().last() {
                Some(post_id) => {
                    match post_id.parse::<i16>() {
                        Ok(post_id) => {
                            match infos.belongs_to {
                                BelongsTo::BlogPost => {
                                    if !services::blog::articles::exists(&pool, post_id).await {
                                        return HttpResponse::NotFound().finish()
                                    }
        
                                    id = Some(post_id);
                                },
                                BelongsTo::Project => {
                                    if !services::projects::exists(&pool, post_id).await {
                                        return HttpResponse::NotFound().finish()
                                    }

                                    id = Some(post_id)
                                },
                                _ => ()
                            }
                        },
                        _ => return HttpResponse::InternalServerError().finish()
                    }

                },
                _ => return HttpResponse::InternalServerError().finish()
            }
        }
    }

    if let Ok(ua) = UserAgent::from_request(&req, &mut actix_web::dev::Payload::None).await {
        let sid = match Uuid::from_str(infos.sid.as_str()) {
            Ok(val) => val,
            Err(_) =>  return HttpResponse::BadRequest().finish()
        };

        let digest_ip = digest::digest(&digest::SHA256, req.peer_addr().unwrap().ip().to_string().as_bytes());
        let digest_ip = format!("{:?}", digest_ip);

        if let Ok(metric_id) = services::metrics::add(
            &pool,
            match infos.belongs_to {
                BelongsTo::Project => services::metrics::BelongsTo::Project(id.unwrap()),
                BelongsTo::BlogPost => services::metrics::BelongsTo::BlogPost(id.unwrap()),
                BelongsTo::Page => services::metrics::BelongsTo::Page(id.unwrap()),
            },
            Some(sid),
            &digest_ip,
            ua.name.clone(),
            ua.os.clone(),
            ua.category.clone(),
            match req.headers().get(actix_web::http::header::REFERER) {
                Some(referer) => match referer.to_str() {
                    Ok(referer) => Some(referer.to_string()),
                    _ => None,
                },
                _ => None,
            },
        )
        .await {
            return HttpResponse::Ok().body(metric_id.to_hyphenated().to_string())
        }
    }

    HttpResponse::NotFound().finish()
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
    let token = if let Ok(token) = Uuid::from_str(&form.token) {
        token
    } else {
        return HttpResponse::BadRequest().finish()
    };
    let session_id: Option<Uuid> = match &form.session_id {
        Some(val) => match Uuid::from_str(val.as_str()) {
                Ok(res) => Some(res),
                Err(_) => None
            }
        None => None
    };

    if !services::metrics::exists(&pool, token).await {
        return HttpResponse::NotFound().finish()
    }

    match services::metrics::update_end_date(&pool, session_id, token).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
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
    let digest_ip = digest::digest(&digest::SHA256, req.peer_addr().unwrap().ip().to_string().as_bytes());
    let digest_ip = format!("{:?}", digest_ip);
    
    if let Ok(session_data) = services::metrics::sessions::add(
        &pool,
        &digest_ip
    )
    .await {
        let sid = session_data.0.to_hyphenated().to_string();
        let vud = session_data.1;
        return HttpResponse::Ok().json(SessionData {
            sid,
            vud
        })
    }

    HttpResponse::InternalServerError().finish()
}