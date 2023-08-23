use chrono::{DateTime, Utc};
use sqlx::{types::Uuid, PgPool};

pub async fn add(pool: &PgPool, ip: &str) -> Result<(Uuid, DateTime<Utc>), sqlx::Error> {
    let res = sqlx::query!(
        "INSERT INTO metric_sessions (ip) VALUES ($1) RETURNING id as sid, expiration_date as vud",
        ip
    )
    .fetch_one(pool)
    .await?;

    Ok((res.sid, res.vud))
}
