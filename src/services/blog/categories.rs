use serde::Deserialize;
use sqlx::{Error, PgPool};

#[derive(Deserialize, Debug)]
pub struct CategoryInformations {
    name: String,
    description: Option<String>,
    is_visible: Option<bool>,
    is_seo: Option<bool>,
    order: i16,
}

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM blog_categories WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn exists_for_uri(pool: &PgPool, uri: &str) -> bool {
    sqlx::query!("SELECT 1 AS one FROM blog_categories WHERE uri = $1", uri)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn get<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    id: i16,
) -> Result<T, Error> {
    let category = sqlx::query_as::<_, T>(&format!(
        "SELECT {} FROM blog_categories WHERE id = $1 LIMIT 1",
        fields
    ))
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(category)
}

pub async fn get_all<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    is_visible: Option<bool>,
    is_seo: Option<bool>,
) -> Vec<T> {
    let is_visible = is_visible.unwrap_or(true);
    let is_seo = is_seo.unwrap_or(true);
    let query = format!(
        r#"SELECT {} FROM blog_categories WHERE is_visible = $1 AND is_seo = $2 ORDER BY "order""#,
        fields
    );

    sqlx::query_as::<_, T>(&query)
        .bind(is_visible)
        .bind(is_seo)
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

pub async fn update(
    pool: &PgPool,
    id: i16,
    category: &CategoryInformations,
) -> Result<bool, Error> {
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
        .rows_affected()
        == 1
}
