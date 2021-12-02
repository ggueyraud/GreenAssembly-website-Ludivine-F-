use chrono::{DateTime, Utc};

pub mod articles;
pub mod categories;

#[derive(sqlx::FromRow)]
pub struct Category {
    pub id: i16,
    pub name: String,
    pub description: Option<String>,
    pub is_visible: Option<bool>,
    pub is_seo: Option<bool>,
}

pub struct Article {
    pub id: i16,
    pub category: Option<serde_json::Value>,
    // cover:
    pub title: String,
    pub date: DateTime<Utc>,
    pub is_published: Option<bool>,
    pub is_seo: Option<bool>,
}
// TODO : update_uri