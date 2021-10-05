use sqlx::{Error, PgPool};

// TODO : count under ten minutes or one hour?
pub async fn count(pool: &PgPool, ip: &str, is_login: bool) -> i64 {
    sqlx::query!(
        "SELECT
            COUNT(id)
        FROM attempts
        WHERE ip = $1 AND is_login = $2 AND date >= NOW() - interval '1 hour'",
        ip,
        is_login
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .count
    .unwrap()
}

pub async fn add(pool: &PgPool, email: &str, ip: &str, is_login: bool) -> Result<i16, Error> {
    let res = sqlx::query!(
        "INSERT INTO attempts (email, ip, is_login)
        VALUES ($1, $2, $3)
        RETURNING id",
        email,
        ip,
        is_login
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn clear(pool: &PgPool, ip: &str) -> bool {
    let rows = sqlx::query!("DELETE FROM attempts WHERE ip = $1", ip)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}
