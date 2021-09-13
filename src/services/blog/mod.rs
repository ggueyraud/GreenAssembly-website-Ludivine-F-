use chrono::{DateTime, Utc};

pub mod articles;
pub mod categories;

pub struct Category {
    id: i16,
    name: String,
    description: Option<String>,
    is_visible: Option<bool>,
    is_seo: Option<bool>
}

pub struct Article {
    id: i16,
    category: Option<serde_json::Value>,
    // cover: 
    name: String,
    content: String,
    date: DateTime<Utc>,
    is_published: bool,
    is_seo: bool
}