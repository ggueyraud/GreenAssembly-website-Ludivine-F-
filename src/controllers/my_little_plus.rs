use super::metrics;
use crate::services;
use actix_extract_multipart::*;
use actix_identity::Identity;
use actix_web::{delete, get, patch, post, put, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::PgPool;
use std::{collections::HashSet, path::Path};

#[get("/mes-petits-plus")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "mes-petits-plus").await {
        let (metric_id, videos) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::videos::get_all(&pool)
        );

        let mut token: Option<String> = None;
        if let Ok(Some(id)) = metric_id {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        #[derive(Template)]
        #[template(path = "pages/my-little-plus.html")]
        struct MyLittlePlus {
            images: Vec<String>, // String path
            title: String,
            description: Option<String>,
            year: i32,
            metric_token: Option<String>,
        }

        return MyLittlePlus {
            images: Vec::new(),
            title: page.title,
            description: page.description,
            year: chrono::Utc::now().year(),
            metric_token: token,
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}
