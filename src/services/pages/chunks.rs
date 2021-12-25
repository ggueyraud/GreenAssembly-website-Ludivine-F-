use sqlx::{Error, PgPool};

pub async fn get<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(pool: &PgPool, fields: &str, identifier: &str) -> Result<T, Error> {
    sqlx::query_as::<_, T>(&format!(
        "SELECT {} FROM page_chunks WHERE identifier = $1",
        fields
    ))
    .bind(identifier)
    .fetch_one(pool)
    .await
}

// pub async fn update(
//     pool: &PgPool,
//     identifier: &str,
//     page_id: i16,
//     content: String
// ) -> Result<bool, Error> {
//     let res = sqlx::query!(
//         "UPDATE page_chunks SET content = jsonb_set(content, 'link', '$1') WHERE identifier = $2 AND page_id",
//         content
//     )
//     .execute(pool)
//     .await?;

//     Ok(res.rows_affected)
// }