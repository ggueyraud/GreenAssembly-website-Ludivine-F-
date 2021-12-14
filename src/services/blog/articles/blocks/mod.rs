pub mod images;

use serde_json::Value;
use sqlx::{Error, PgPool};
use std::collections::HashMap;

pub async fn get_all<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    article_id: i16,
) -> Vec<T> {
    sqlx::query_as::<_, T>(&format!(
        r#"SELECT {} FROM blog_article_blocks WHERE article_id = $1 ORDER BY "order""#,
        fields
    ))
    .bind(article_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn insert(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    article_id: i16,
    title: Option<&str>,
    content: Option<&str>,
    left_column: bool,
    order: i16,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO blog_article_blocks
            (article_id, title, content, left_column, "order")
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id"#,
        article_id,
        title,
        content,
        left_column,
        order
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn partial_update(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: i16,
    fields: HashMap<String, serde_json::Value>,
) -> Result<bool, Error> {
    if fields.len() > 0 {
        let mut query = String::from("UPDATE blog_article_blocks SET");
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
                },
                Value::Bool(value) => query = query.bind(value),
                _ => (),
            }
        }

        let res = query.bind(id).execute(pool).await?;

        return Ok(res.rows_affected() == 1);
    }

    Ok(false)
}

pub async fn delete(pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>, id: i16) -> bool {
    sqlx::query!("DELETE FROM blog_article_blocks WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
