use sqlx::{types::Uuid, Error, PgPool};

#[derive(sqlx::FromRow)]
pub struct BlogArticleImage {
    pub id: Uuid,
    pub path: String,
}

pub async fn get_all(pool: &PgPool, article_id: i16) -> Vec<BlogArticleImage> {
    sqlx::query_as!(
        BlogArticleImage,
        r#"SELECT
            bai.id AS "id",
            f.path AS "path"
        FROM blog_article_images bai
        JOIN files f ON bai.file_id = f.id
        WHERE bai.article_id = $1"#,
        article_id
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn insert(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    article_id: i16,
    file_id: i32,
) -> Result<Uuid, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO blog_article_images
            (article_id, file_id)
        VALUES ($1, $2)
        RETURNING id"#,
        article_id,
        file_id
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn delete(pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>, id: Uuid) -> bool {
    sqlx::query!("DELETE FROM blog_article_images WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
