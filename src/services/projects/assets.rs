use sqlx::{Error, PgPool};

#[derive(sqlx::FromRow)]
pub struct Asset {
    id: i16,
    pub project_id: i16,
    pub path: String,
    pub order: i16,
    pub is_visible: bool,
}

pub async fn get(pool: &PgPool, id: i16) -> Result<Asset, Error> {
    sqlx::query_as!(
        Asset,
        r#"SELECT $1::int2 AS "id!: i16", project_id, path, "order", is_visible FROM project_assets WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM project_assets WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn get_all(pool: &PgPool, project_id: i16) -> Vec<super::Asset> {
    sqlx::query_as!(
        super::Asset,
        r#"SELECT id, path, "order", is_visible FROM project_assets WHERE project_id = $1"#,
        project_id
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn insert(
    pool: &PgPool,
    project_id: i16,
    path: &str,
    order: i16,
    is_visible: bool,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO project_assets
            (project_id, path, "order", is_visible)
        VALUES ($1, $2, $3, $4)
        RETURNING id"#,
        project_id,
        path,
        order,
        is_visible
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update(pool: &PgPool, id: i16, order: i16, is_visible: bool) -> Result<bool, Error> {
    let res = sqlx::query!(
        r#"UPDATE project_assets SET
            "order" = $1,
            is_visible = $2
        WHERE id = $3"#,
        order,
        is_visible,
        id
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    let rows = sqlx::query!("DELETE FROM project_assets WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}
