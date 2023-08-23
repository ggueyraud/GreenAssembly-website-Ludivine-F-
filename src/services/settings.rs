use serde_json::Value;
use sqlx::{Error, PgPool};
use std::collections::HashMap;

#[derive(sqlx::FromRow, Debug)]
pub struct Settings {
    pub background_color: String,
    pub title_color: String,
    pub text_color: String,
}

pub async fn get(pool: &PgPool) -> Result<Settings, Error> {
    sqlx::query_as!(Settings, "SELECT * FROM settings")
        .fetch_one(pool)
        .await
}

pub async fn partial_update(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    fields: HashMap<String, Value>,
) -> Result<bool, Error> {
    if !fields.is_empty() {
        let mut query = String::from("UPDATE settings SET ");

        for (i, (key, _)) in fields.iter().enumerate() {
            let index = i + 1;

            if index > 1 {
                query += ",";
            }

            query += &format!(r#""{}" = ${}"#, key, index);

            if i == fields.len() {
                query += &format!(" WHERE id = ${}", index);
            }
        }

        println!("Query : {} | Fields: {:?}", query, fields);
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

        let res = query.execute(pool).await?;

        return Ok(res.rows_affected() == 1);
    }

    Ok(false)
}
