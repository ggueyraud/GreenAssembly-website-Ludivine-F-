use sqlx::PgPool;

pub async fn exist_for_email(pool: &PgPool, email: &str) -> bool {
    sqlx::query!(r#"SELECT 1 AS one FROM "user" WHERE email = $1"#, email)
        .fetch_one(pool)
        .await
        .is_ok()
}

// pub async fn update(pool: &PgPool, )
