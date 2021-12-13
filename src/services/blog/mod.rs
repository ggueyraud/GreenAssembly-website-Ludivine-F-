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