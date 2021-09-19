use sqlx::{Error, PgPool, types::Uuid};

pub mod tokens;

pub enum BelongsTo {
    Page(i16),
    Project(i16),
}

pub async fn add(
    pool: &PgPool,
    belongs_to: BelongsTo,
    ip: &str,
    browser: Option<String>,
    os: Option<String>,
    device_type: Option<String>,
    referer: Option<String>,
) -> Result<i32, Error> {
    use sqlx::Row;

    let mut id: Option<i16> = None;
    let query = &format!(
        "INSERT INTO metrics ({}, ip, browser, os, device_type, referer)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id",
        match belongs_to {
            BelongsTo::Page(page_id) => {
                id = Some(page_id);
                "page_id"
            }
            BelongsTo::Project(project_id) => {
                id = Some(project_id);
                "project_id"
            }
        }
    );

    let res = sqlx::query(query)
        .bind(id.unwrap())
        .bind(ip)
        .bind(browser)
        .bind(os)
        .bind(device_type)
        .bind(referer)
        .fetch_one(pool)
        .await?;
    let id = res.try_get("id")?;

    Ok(id)
}

pub async fn close(pool: &PgPool, id: i32) -> Result<bool, Error> {
    let res = sqlx::query!("UPDATE metrics SET end_date = NOW() WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() == 1)
}