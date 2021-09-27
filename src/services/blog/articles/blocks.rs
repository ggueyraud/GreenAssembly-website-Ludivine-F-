use sqlx::PgPool;

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
