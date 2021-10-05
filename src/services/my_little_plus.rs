use sqlx::PgPool;

pub async fn patch_image(pool: &PgPool, id: i16, file_id: i32) -> bool {
    sqlx::query!(
        "UPDATE my_little_plus_images SET file_id = $2 WHERE id = $1",
        id,
        file_id
    )
    .fetch_one(pool)
    .await
    .is_ok()
}

pub async fn get_images(pool: &PgPool) -> Vec<String> {
    match sqlx::query!(
        "
        SELECT f.path FROM my_little_plus_images mlpi
        LEFT JOIN files f ON f.id = mlpi.file_id
    "
    )
    .fetch_all(pool)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|resp| match resp.path {
                Some(path) => path,
                None => "".to_owned(),
            })
            .collect(),
        Err(_) => vec!["".to_owned(); 7],
    }
}
