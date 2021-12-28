use serde_json::Value;
use sqlx::{Error, PgPool};

pub mod images;

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

// pub async fn get_id_by_uri(
//     pool: &PgPool,
//     uri: &str
// ) -> Result<i16, Error> {
//     let article = sqlx::query!(
//         "SELECT
//             id
//         FROM blog_articles
//         WHERE uri = $1
//         LIMIT 1",
//         uri
//     )
//     .fetch_one(pool)
//     .await?;

//     Ok(article.id)
// }

pub async fn get_all<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    is_published: Option<bool>,
    is_seo: Option<bool>,
    category_id: Option<i16>,
) -> Vec<T> {
    // let is_published = is_published.unwrap_or(true);
    // let is_seo = is_seo.unwrap_or(true);
    let mut query = format!(
        "SELECT {}
        FROM blog_articles ba
        JOIN files f ON ba.cover_id = f.id
        WHERE (($1 IS NOT NULL AND ba.is_published = $1) OR $1 IS NULL)
        AND (($2 IS NOT NULL AND ba.is_seo = $2) OR $2 IS NULL)",
        fields
    );

    if category_id.is_some() {
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

pub async fn insert(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    category_id: Option<i16>,
    cover_id: i32,
    title: &str,
    description: Option<&str>,
    content: &str,
    is_published: Option<bool>,
    is_seo: Option<bool>,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        "INSERT INTO blog_articles
            (category_id, cover_id, title, description, content, is_published, is_seo)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id",
        category_id,
        cover_id,
        title,
        description,
        content,
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
    if !fields.is_empty() {
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
