use sqlx::{Error, FromRow, PgPool};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Links {
    pub creations: Option<String>,
    pub shootings: Option<String>,
}

pub async fn get_links(pool: &PgPool) -> Option<Links> {
    match sqlx::query_as!(
        Links,
        "SELECT
            creations,
            shootings
        FROM my_little_plus_links"
    )
    .fetch_one(pool)
    .await {
        Ok(val) => Some(val),
        Err(_) => None
    }
}

pub async fn edit_links(pool: &PgPool, links: &Links) -> Result<(), Error> {
    match sqlx::query!(
        "UPDATE
            my_little_plus_links
        SET
            creations = COALESCE($1, creations),
            shootings = COALESCE($2, shootings)
        ",
        links.creations,
        links.shootings
    )
    .execute(pool)
    .await {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}
