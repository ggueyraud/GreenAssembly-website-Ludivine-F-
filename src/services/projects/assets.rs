use sqlx::{Error, PgPool};

pub async fn count(pool: &PgPool, project_id: i16) -> i64 {
    sqlx::query!(
        "SELECT
            COUNT(id)
        FROM project_assets
        WHERE project_id = $1",
        project_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .count
    .unwrap()
}

pub async fn get<
    T: std::marker::Unpin + std::marker::Send + for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
>(
    pool: &PgPool,
    fields: &str,
    id: i16,
) -> Result<T, Error> {
    sqlx::query_as::<_, T>(
        &format!(
            "SELECT
                {}
            FROM projects_assets pa
            JOIN files f ON f.id = pa.file_id
            WHERE pa.id = $1",
            fields
        )
        // r#"SELECT
        //     $1::int2 AS "id!: i16", pa.project_id AS "project_id", f.path AS "path", pa.order AS "order"
        // FROM project_assets pa
        // JOIN files f ON f.id = pa.file_id
        // WHERE pa.id = $1"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_available_slots(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    project_id: i16,
) -> Vec<i16> {
    let rows = sqlx::query!(
        r#"SELECT "order" FROM project_assets WHERE project_id = $1"#,
        project_id
    )
    .fetch_all(pool)
    .await
    .unwrap();

    let mut available_slots = vec![0, 1, 2, 3, 4];

    for row in &rows {
        if let Some(index) = available_slots.iter().position(|x| x == &row.order) {
            available_slots.remove(index);
        }
    }

    available_slots
}

pub async fn get_all(pool: &PgPool, project_id: i16) -> Vec<super::Asset> {
    sqlx::query_as!(
        super::Asset,
        r#"SELECT
            pa.id AS "id", f.path AS "path"
        FROM project_assets pa
        JOIN files f ON f.id = pa.file_id
        WHERE project_id = $1
        ORDER BY pa.order"#,
        project_id
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn insert(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    project_id: i16,
    file_id: i32,
    order: i16,
) -> Result<i16, Error> {
    let res = sqlx::query!(
        r#"INSERT INTO project_assets
            (project_id, file_id, "order")
        VALUES ($1, $2, $3)
        RETURNING id"#,
        project_id,
        file_id,
        order
    )
    .fetch_one(pool)
    .await?;

    Ok(res.id)
}

pub async fn update(
    pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: i16,
    order: i16,
) -> Result<bool, Error> {
    let res = sqlx::query!(
        r#"UPDATE project_assets SET
            "order" = $1
        WHERE id = $2"#,
        order,
        id
    )
    .execute(pool)
    .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn delete(pool: impl sqlx::Executor<'_, Database = sqlx::Postgres>, id: i16) -> bool {
    sqlx::query!("DELETE FROM project_assets WHERE id = $1", id)
        .execute(pool)
        .await
        .unwrap()
        .rows_affected()
        == 1
}
