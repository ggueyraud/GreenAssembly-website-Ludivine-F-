use sqlx::{Error, PgPool};

pub async fn insert(pool: &PgPool, name: Option<&str>, path: Option<&str>) -> Result<i32, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO files (name, path) VALUES ($1, $2) RETURNING id"#,
        name,
        path
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update(pool: &PgPool, id: i32, name: Option<&str>) -> Result<bool, Error> {
    let res = sqlx::query!("UPDATE files SET name = $1 WHERE id = $2", name, id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() == 1)
}
