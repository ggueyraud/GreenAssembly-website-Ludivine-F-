use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
    pub content: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: i16,
    pub path: String,
}

pub async fn get_all(pool: &PgPool, category_id: Option<i16>) -> Vec<Project> {
    let mut query = String::from(
        "SELECT
            id, name, content, date
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

#[derive(Deserialize, Debug)]
pub struct ProjectInformations {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
}

pub async fn insert(pool: &PgPool, project: &ProjectInformations) -> Result<i16, Error> {
    let res = sqlx::query!(
        "INSERT INTO projects
            (name, description, content)
        VALUES ($1, $2, $3)
        RETURNING id",
        project.name,
        project.description,
        project.content
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update(pool: &PgPool, id: i16, project: &ProjectInformations) -> Result<bool, Error> {
    let res = sqlx::query!(
        r#"UPDATE projects SET
            name = $1,
            description = $2,
            content = $3
        WHERE id = $4"#,
        project.name,
        project.description,
        project.content,
        id
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    let rows = sqlx::query!("DELETE FROM projects WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}
