use sqlx::{Error, FromRow, PgPool};

pub mod chunks;

#[derive(FromRow)]
pub struct Page {
    pub id: i16,
    pub title: String,
    pub description: Option<String>,
}

pub async fn get(pool: &PgPool, identifier: &str) -> Result<Page, Error> {
    sqlx::query_as!(
        Page,
        "SELECT
            id, title, description
        FROM pages
        WHERE identifier = $1
        LIMIT 1",
        identifier
    )
    .fetch_one(pool)
    .await
}
