use sqlx::{Postgres, Executor, Error};

pub async fn get_all(
    pool: impl Executor<'_, Database = Postgres>,
    block_id: i16
) -> Vec<String> {
    sqlx::query!(
        "SELECT
            f.path
        FROM blog_article_block_images babi
        JOIN files f ON f.id = babi.file_id
        WHERE block_id = $1",
        block_id
    )
        .fetch_all(pool)
        .await
        .unwrap()
        .iter()
        .map(|row| row.path.clone())
        .collect::<Vec<String>>()
}

pub async fn insert(
    pool: impl Executor<'_, Database = Postgres>,
    block_id: i16,
    file_id: i32
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO blog_article_block_images
            (block_id, file_id)
        VALUES ($1, $2)",
        block_id,
        file_id
    )
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete(
    pool: impl Executor<'_, Database = Postgres>,
    block_id: i16
) -> bool {
    sqlx::query!("DELETE FROM blog_article_block_images WHERE block_id = $1", block_id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}