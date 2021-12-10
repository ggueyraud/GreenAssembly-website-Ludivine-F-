use sqlx::{Error, PgPool};

pub async fn insert(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    name: Option<&str>,
    path: &str,
) -> Result<i32, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO files (name, path) VALUES ($1, $2) RETURNING id"#,
        name,
        path
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn get<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    id: i32,
    fields: &str,
) -> Result<T, Error> {
    sqlx::query_as::<_, T>(&format!(
        "SELECT
            {}
        FROM files
        WHERE id = $1",
        fields
    ))
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>, id: i32) -> bool {
    let rows = sqlx::query!("DELETE FROM files WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}
