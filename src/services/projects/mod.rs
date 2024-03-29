use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{Error, FromRow, PgPool};

pub mod assets;
pub mod categories;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Category {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Project {
    pub id: i16,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: i16,
    pub path: String,
}

pub async fn get_all(pool: &PgPool, category_id: Option<i16>) -> Vec<Project> {
    let query = String::from(
        "SELECT
            id, name, description, content, date
        FROM projects
        ORDER BY date DESC",
    );

    // if category_id.is_some() {
    //     query += " WHERE category_id = $1"
    // }

    let mut projects = sqlx::query_as::<_, Project>(&query);

    if let Some(category_id) = category_id {
        projects = projects.bind(category_id);
    }

    projects.fetch_all(pool).await.unwrap()
}

pub async fn get_all_spe<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    category_id: Option<i16>,
) -> Result<Vec<T>, Error> {
    let query = format!(
        "SELECT
            {}
        FROM projects
        ORDER BY date DESC",
        fields
    );

    // if category_id.is_some() {
    //     query += " WHERE category_id = $1"
    // }

    let mut projects = sqlx::query_as::<_, T>(&query);

    if let Some(category_id) = category_id {
        projects = projects.bind(category_id);
    }

    let res = projects.fetch_all(pool).await?;

    Ok(res)
}

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM projects WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

#[derive(Debug)]
pub struct ProjectDetails {
    pub id: i16,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub date: DateTime<Utc>,
}

pub async fn get(pool: &PgPool, id: i16) -> Result<ProjectDetails, Error> {
    let project = sqlx::query!(
        "SELECT
            name, description, content, date
        FROM projects
        WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await?;

    // let (project, assets) = futures::join!(project, assets);
    // let project = project?;
    // let assets = assets?;

    Ok(ProjectDetails {
        id,
        name: project.name,
        description: project.description,
        content: project.content,
        date: project.date,
        // assets,
    })
}

pub async fn get_spe<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    id: i16,
) -> Result<T, Error> {
    let query = format!("SELECT {} FROM projects WHERE id = $1 LIMIT 1", fields);

    let res = sqlx::query_as::<_, T>(&query)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(res)
}

pub async fn insert(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    name: &str,
    description: Option<&str>,
    content: &str,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        "INSERT INTO projects
            (name, description, content)
        VALUES ($1, $2, $3)
        RETURNING id",
        name,
        description,
        content
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
        let mut query = String::from("UPDATE projects SET ");
        let mut i = 1;

        for (key, _) in fields.iter() {
            if i > 1 {
                query += ",";
            }

            query += &format!(r#"{} = ${}"#, key, i);

            i += 1;
        }

        query += &format!(" WHERE id = ${}", i);

        let mut query = sqlx::query(&query);

        for (_, value) in fields.iter() {
            match value {
                Value::String(value) => query = query.bind(value.as_str()),
                Value::Null => query = query.bind(Option::<bool>::None),
                _ => (),
            }
        }

        let res = query.bind(id).execute(pool).await?;

        return Ok(res.rows_affected() == 1);
    }

    Ok(false)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    let rows = sqlx::query!("DELETE FROM projects WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}

pub async fn link_to_category(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    project_id: i16,
    category_id: i16,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO projects_categories
            (project_id, category_id)
        VALUES ($1, $2)",
        project_id,
        category_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn detach_categories(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    project_id: i16,
) -> bool {
    let rows = sqlx::query!(
        "DELETE FROM projects_categories WHERE project_id = $1",
        project_id
    )
    .execute(pool)
    .await
    .unwrap()
    .rows_affected();

    rows == 1
}
