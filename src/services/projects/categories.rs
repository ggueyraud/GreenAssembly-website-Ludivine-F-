use sqlx::{Error, PgPool};

pub async fn exists(pool: &PgPool, id: i16) -> bool {
    sqlx::query!("SELECT 1 AS one FROM project_categories WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn get_all(pool: &PgPool) -> Vec<super::Category> {
    sqlx::query_as!(
        super::Category,
        r#"SELECT id, name FROM project_categories ORDER BY "order""#
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn insert(pool: &PgPool, name: &str, order: i16) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO project_categories
            (name, "order")
        VALUES ($1, $2)
        RETURNING id"#,
        name,
        order
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update(pool: &PgPool, id: i16, name: &str, order: i16) -> Result<bool, Error> {
    let res = sqlx::query!(
        r#"UPDATE project_categories SET
            name = $1,
            "order" = $2
        WHERE id = $3"#,
        name,
        order,
        id
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn delete(pool: &PgPool, id: i16) -> bool {
    let rows = sqlx::query!("DELETE FROM project_categories WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    rows == 1
}
