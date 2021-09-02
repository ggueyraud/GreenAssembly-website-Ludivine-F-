use sqlx::{Error, PgPool};

// TODO : count under ten minutes or one hour?
pub async fn count(pool: &PgPool, ip: &str) -> i64 {
    sqlx::query!(
        "SELECT
            COUNT(id)
        FROM login_attempts
        WHERE ip = $1",
        ip
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .count
    .unwrap()
}

pub async fn add(
    pool: &PgPool,
    email: &str,
    ip: &str,
    browser: Option<&str>,
    os: Option<&str>,
    device_type: Option<&str>,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        "INSERT INTO login_attempts (email, ip, browser, os, device_type)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id",
        email,
        ip,
        browser,
        os,
        device_type
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn clear(pool: &PgPool, ip: &str) -> bool {
    let rows = sqlx::query!("DELETE FROM login_attempts WHERE ip = $1", ip)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}
