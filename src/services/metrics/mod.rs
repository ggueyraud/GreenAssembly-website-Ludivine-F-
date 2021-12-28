use sqlx::{Error, PgPool, types::Uuid};

pub mod sessions;

pub enum BelongsTo {
    Page(i16),
    Project(i16),
    BlogPost(i16),
}

pub async fn exists(pool: &PgPool, id: Uuid) -> bool {
    sqlx::query!(
        "SELECT 1 AS one FROM metrics WHERE id = $1", id
    )
        .fetch_one(pool)
        .await
        .is_ok()
}

pub async fn add(
    pool: &PgPool,
    belongs_to: BelongsTo,
    session_id: Option<Uuid>,
    ip: &str,
    browser: Option<String>,
    os: Option<String>,
    device_type: Option<String>,
    referer: Option<String>,
) -> Result<Uuid, Error> {
    use sqlx::Row;

    let mut id: Option<i16> = None;

    let query = &format!(
        "INSERT INTO metrics ({}, session_id, ip, browser, os, device_type, referer)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
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
            BelongsTo::BlogPost(post_id) => {
                id = Some(post_id);
                "post_id"
            }
        }
    );

    let res = sqlx::query(query)
        .bind(id.unwrap())
        .bind(session_id)
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

pub async fn update_end_date(pool: &PgPool, session_id: Option<Uuid>, id: Uuid) -> Result<bool, Error> {
    let res = sqlx::query!(
            "UPDATE metrics
            SET end_date = NOW(),
                session_id = $1
            WHERE id = $2",
            session_id,
            id
        )
        .execute(pool)
        .await?;

    Ok(res.rows_affected() == 1)
}