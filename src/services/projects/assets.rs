use sqlx::{Error, PgPool};

pub async fn count(pool: &PgPool, project_id: i16) -> i64 {
    sqlx::query!(
        "SELECT
            COUNT(id)
        FROM project_assets
        WHERE project_id = $1",
        project_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .count
    .unwrap()
}

#[derive(sqlx::FromRow)]
pub struct Asset {
    id: i16,
    pub project_id: i16,
    pub path: String,
    pub order: i16,
}

pub async fn get(pool: &PgPool, id: i16) -> Result<Asset, Error> {
    sqlx::query_as!(
        Asset,
        r#"SELECT
            $1::int2 AS "id!: i16", pa.project_id AS "project_id", f.path AS "path", pa.order AS "order"
        FROM project_assets pa
        JOIN files f ON f.id = pa.file_id
        WHERE pa.id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn exists(pool: &PgPool, project_id: i16, asset_id: i16) -> bool {
    sqlx::query!(
        "SELECT 1 AS one FROM project_assets WHERE id = $1 AND project_id = $2",
        asset_id,
        project_id
    )
    .fetch_one(pool)
    .await
    .is_ok()
}

pub async fn get_all(pool: &PgPool, project_id: i16) -> Vec<super::Asset> {
    sqlx::query_as!(
        super::Asset,
        r#"SELECT
            pa.id AS "id", f.path AS "path"
        FROM project_assets pa
        JOIN files f ON f.id = pa.file_id
        WHERE project_id = $1
        ORDER BY pa.order"#,
        project_id
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn insert(
    pool: &PgPool,
    project_id: i16,
    file_id: i32,
    order: i16,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO project_assets
            (project_id, file_id, "order")
        VALUES ($1, $2, $3)
        RETURNING id"#,
        project_id,
        file_id,
        order
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: i16,
    order: i16
) -> Result<bool, Error> {
    let res = sqlx::query!(
        r#"UPDATE project_assets SET
            "order" = $1
        WHERE id = $2"#,
        order,
        id
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn delete(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: i16
) -> bool {
    sqlx::query!("DELETE FROM project_assets WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
