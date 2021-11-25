use sqlx::{Error, PgPool};

pub async fn get_all<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    article_id: i16,
) -> Vec<T> {
    sqlx::query_as::<_, T>(&format!(
        "SELECT {} FROM blog_article_blocks WHERE article_id = $1",
        fields
    ))
    .bind(article_id)
    .fetch_all(pool)
    .await
    .unwrap()
}


pub async fn insert(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    article_id: i16,
    title: Option<&str>,
    content: Option<&str>,
    left_column: bool,
    order: i16
) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO blog_article_blocks
            (article_id, title, content, left_column, "order")
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id"#,
        article_id,
        title,
        content,
        left_column,
        order
    )
        .fetch_one(pool)
        .await?;

    Ok(res.id)
}