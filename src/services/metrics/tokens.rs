use sqlx::{types::Uuid, Error, PgPool};

pub async fn add(pool: &PgPool, metric_id: i32) -> Result<Uuid, Error> {
    let res = sqlx::query!(
        "INSERT INTO metric_tokens (metric_id) VALUES ($1) RETURNING token",
        metric_id
    )
    .fetch_one(pool)
    .await?;

    Ok(res.token)
}

pub async fn get_metric(pool: &PgPool, token: Uuid) -> Result<i32, Error> {
    let res = sqlx::query!(
        "SELECT metric_id FROM metric_tokens WHERE token = $1 LIMIT 1",
        token
    )
    .fetch_one(pool)
    .await?;

    Ok(res.metric_id)
}

pub async fn exists(pool: &PgPool, token: Uuid) -> bool {
    sqlx::query!("SELECT 1 AS one FROM metric_tokens WHERE token = $1", token)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn delete(pool: &PgPool, token: Uuid) -> bool {
    sqlx::query!("DELETE FROM metric_tokens WHERE token = $1", token)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
