use sqlx::{Error, PgPool};

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM project_categories WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn get_all(pool: &PgPool, project_id: Option<i16>) -> Vec<super::Category> {
    let mut query = String::from(
        "SELECT
            pc.id, pc.name
        FROM project_categories pc
        LEFT JOIN projects_categories pcs ON pcs.category_id = pc.id",
    );

    if project_id.is_some() {
        query += " WHERE pcs.project_id = $1";
    }

    query += r#" ORDER BY "order""#;

    let mut query = sqlx::query_as::<_, super::Category>(&query);

    if let Some(project_id) = project_id {
        query = query.bind(project_id);
    }

    query.fetch_all(pool).await.unwrap()
}

pub async fn insert(pool: &PgPool, name: &str) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO project_categories
            (name, "order")
        VALUES ($1,  (SELECT COUNT(id) FROM project_categories) + 1)
        RETURNING id"#,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

use serde_json::Value;
use std::collections::HashMap;

pub async fn partial_update(
    pool: &PgPool,
    id: i16,
    fields: HashMap<String, serde_json::Value>,
) -> Result<bool, Error> {
    if !fields.is_empty() {
        let mut query = String::from("UPDATE project_categories SET ");
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
                _ => (),
            }
        }

        let res = query.bind(id).execute(pool).await?;

        return Ok(res.rows_affected() == 1);
    }

    Ok(false)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    let rows = sqlx::query!("DELETE FROM project_categories WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}
