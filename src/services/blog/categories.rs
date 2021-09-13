use serde::Deserialize;
use sqlx::{Error, PgPool};

#[derive(Deserialize, Debug)]
pub struct CategoryInformations {
    name: String,
    description: Option<String>,
    is_visible: Option<bool>,
    is_seo: Option<bool>,
    order: i16
}

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM blog_categories WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn get_all(pool: &PgPool) -> Vec<super::Category> {
    sqlx::query_as!(
        super::Category,
        "SELECT id, name, description, is_visible, is_seo FROM blog_categories"
    )
        .fetch_all(pool)
        .await
        .unwrap()
}

pub async fn insert(pool: &PgPool, category: &CategoryInformations) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO blog_categories
            (name, description, is_visible, is_seo, "order")
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id"#,
        category.name,
        category.description,
        category.is_visible,
        category.is_seo,
        category.order
    )
        .fetch_one(pool)
        .await?;

    Ok(res.id)
}

pub async fn update(pool: &PgPool, id: i16, category: &CategoryInformations) -> Result<bool, Error> {
    let res = sqlx::query!(
        r#"UPDATE blog_categories SET
            name = $1,
            description = $2,
            is_visible = $3,
            is_seo = $4,
            "order" = $5
        WHERE id = $6"#,
        category.name,
        category.description,
        category.is_visible,
        category.is_seo,
        category.order,
        id
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("DELETE FROM blog_categories WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected() == 1
}