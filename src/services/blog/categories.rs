use serde_json::Value;
use sqlx::{Error, PgPool};

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
    let query = format!(
        r#"SELECT
            {}
        FROM blog_categories
        WHERE (($1 IS NOT NULL AND is_visible = $1) OR $1 IS NULL)
        AND (($2 IS NOT NULL AND is_seo = $2) OR $2 IS NULL)
        ORDER BY "order""#,
        fields
    );

    sqlx::query_as::<_, T>(&query)
        .bind(is_visible)
        .bind(is_seo)
        .fetch_all(pool)
        .await
        .unwrap()
}

pub async fn insert(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    is_visible: Option<bool>,
    is_seo: Option<bool>,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO blog_categories
            (name, description, is_visible, is_seo, "order")
        VALUES (
            $1,
            $2,
            $3,
            $4,
            (SELECT COUNT(id) FROM blog_categories) + 1
        )
        RETURNING id"#,
        name,
        description,
        is_visible,
        is_seo,
        // order
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update_uri(pool: &PgPool, id: i16, uri: &str) -> Result<bool, Error> {
    let res = sqlx::query!(
        r#"UPDATE blog_categories SET uri = $1 WHERE id = $2"#,
        uri,
        id,
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn partial_update(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: i16,
    fields: std::collections::HashMap<String, Value>,
) -> Result<bool, Error> {
    if fields.len() > 0 {
        let mut query = String::from("UPDATE blog_categories SET");
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
                Value::String(value) => {
                    if value == "Null" {
                        query = query.bind(Option::<String>::None);
                    } else {
                        query = query.bind(value.as_str());
                    }
                }
                Value::Number(value) => {
                    query = query.bind(value.as_i64());
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
    sqlx::query!("DELETE FROM blog_categories WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
