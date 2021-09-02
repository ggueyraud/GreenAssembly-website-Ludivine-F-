use sqlx::PgPool;

#[derive(Debug)]
pub struct Video {
    id: i16,
    title: String,
    description: Option<String>,
    path: String,
}

pub async fn get_all(pool: &PgPool) -> Vec<Video> {
    sqlx::query_as!(Video, "SELECT id, title, description, path FROM videos")
        .fetch_all(pool)
        .await
        .unwrap()
}
