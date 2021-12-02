use serde::Deserialize;
use serde_json::Value;
use sqlx::{Error, PgPool};

pub mod blocks;

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM blog_articles WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn exists_for_uri(pool: &PgPool, uri: &str) -> bool {
    sqlx::query!("SELECT 1 AS one FROM blog_articles WHERE uri = $1", uri)
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
    let article = sqlx::query_as::<_, T>(&format!(
        "SELECT
            {}
        FROM blog_articles ba
        JOIN files f ON f.id = ba.cover_id
        WHERE ba.id = $1 LIMIT 1",
        fields
    ))
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(article)
}

pub async fn get_all1<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    is_published: Option<bool>,
    is_seo: Option<bool>,
    category_id: Option<i16>,
) -> Vec<T> {
    let is_published = is_published.unwrap_or(true);
    let is_seo = is_seo.unwrap_or(true);
    let mut query = format!(
        "SELECT {}
        FROM blog_articles ba
        JOIN files f ON ba.cover_id = f.id
        WHERE ba.is_published = $1 AND ba.is_seo = $2",
        fields
    );

    if let Some(category_id) = category_id {
        query += " AND ba.category_id = $3"
    }

    query += "ORDER BY ba.id DESC";

    let mut q = sqlx::query_as::<_, T>(&query)
        .bind(is_published)
        .bind(is_seo);

    if let Some(category_id) = category_id {
        q = q.bind(category_id);
    }

    q.fetch_all(pool).await.unwrap()
}

pub async fn get_all(pool: &PgPool) -> Vec<super::Article> {
    sqlx::query!(
        r#"SELECT
            ba.id AS article_id,
            ba.title AS article_title,
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
        title: row.article_title.clone(),
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
    title: String,
    description: Option<String>,
    is_published: bool,
    is_seo: bool,
}

pub async fn insert(
    // pool: &sqlx::Executor<Database = sqlx::Postgres>,
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // pool: &PgPool,
    category_id: Option<i16>,
    cover_id: i32,
    title: &str,
    description: Option<&str>,
    is_published: Option<bool>,
    is_seo: Option<bool>,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        "INSERT INTO blog_articles
            (category_id, cover_id, title, description, is_published, is_seo)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id",
        category_id,
        cover_id,
        title,
        description,
        is_published,
        is_seo
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn partial_update(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: i16,
    fields: std::collections::HashMap<String, serde_json::Value>,
) -> Result<bool, Error> {
    if fields.len() > 0 {
        let mut query = String::from("UPDATE blog_articles SET");
        let mut i = 1;

        for (key, _) in fields.iter() {
            if i > 1 {
                query += ",";
            }

            query += &format!(r#""{}" = ${}"#, key, i);

            i += 1;
        }

        query += &format!(" WHERE id = ${}", i);

        let mut query = sqlx::query(&query);

        for (_, value) in fields.iter() {
            match value {
                Value::Number(value) => {
                    query = query.bind(value.as_i64());
                }
                Value::String(value) => {
                    query = query.bind(value.as_str());
                }
                Value::Bool(value) => query = query.bind(value),
                _ => (),
            }
        }

        let res = query.bind(id).execute(pool).await?;

        return Ok(res.rows_affected() == 1);
    }

    Ok(false)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("DELETE FROM blog_articles WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
