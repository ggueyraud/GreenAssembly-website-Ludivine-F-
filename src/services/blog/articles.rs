use serde::Deserialize;
use sqlx::{Error, PgPool};

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM blog_articles WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn get_all1<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    is_published: Option<bool>,
    is_seo: Option<bool>,
) -> Vec<T> {
    let is_published = is_published.unwrap_or(true);
    let is_seo = is_seo.unwrap_or(true);
    let query = format!(
        "SELECT {}
        FROM blog_articles ba
        LEFT JOIN files f ON ba.cover_id = f.id
        WHERE ba.is_published = $1 AND ba.is_seo = $2
        ORDER BY ba.id DESC",
        fields
    );

    sqlx::query_as::<_, T>(&query)
        .bind(is_published)
        .bind(is_seo)
        .fetch_all(pool)
        .await
        .unwrap()
}

pub async fn get_all(pool: &PgPool) -> Vec<super::Article> {
    sqlx::query!(
        r#"SELECT
            ba.id AS article_id,
            ba.name AS article_name,
            ba.content AS article_content,
            ba.date AS article_date,
            ba.is_published AS article_is_published,
            ba.is_seo AS article_is_seo,
            bc.id AS "category_id?",
            bc.name AS "category_name?"
        FROM blog_articles ba
        LEFT JOIN blog_categories bc ON ba.category_id = bc.id
        ORDER BY ba.id DESC"#
    )
    .fetch_all(pool)
    .await
    .expect("blog::articles::get_all")
    .iter()
    .map(|row| super::Article {
        id: row.article_id,
        category: if let Some(category_id) = row.category_id {
            Some(serde_json::json!({
                "id": category_id,
                "name": row.category_name.as_ref().unwrap()
            }))
        } else {
            None
        },
        name: row.article_name.clone(),
        content: row.article_content.clone(),
        date: row.article_date,
        is_published: row.article_is_published,
        is_seo: row.article_is_seo,
    })
    .collect::<Vec<super::Article>>()
}

#[derive(Deserialize)]
pub struct ArticleInformations {
    category_id: Option<i16>,
    cover_id: Option<i32>,
    name: String,
    content: String,
    is_published: bool,
    is_seo: bool,
}

pub async fn insert(pool: &PgPool, article: &ArticleInformations) -> Result<i16, Error> {
    let res = sqlx::query!(
        "INSERT INTO blog_articles
            (category_id, cover_id, name, content, is_published, is_seo)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id",
        article.category_id,
        article.cover_id,
        article.name,
        article.content,
        article.is_published,
        article.is_seo
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update(pool: &PgPool, id: i16, article: &ArticleInformations) -> Result<bool, Error> {
    let res = sqlx::query!(
        "UPDATE blog_articles SET
            category_id = $1,
            cover_id = $2,
            name = $3,
            content = $4,
            is_published = $5,
            is_seo = $6
        WHERE id = $7",
        article.category_id,
        article.cover_id,
        article.name,
        article.content,
        article.is_published,
        article.is_seo,
        id
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("DELETE FROM blog_articles WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
