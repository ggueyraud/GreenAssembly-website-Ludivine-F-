use super::metrics;
use crate::services;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use sqlx::PgPool;

#[get("")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "portfolio").await {
        use slugmin::slugify;

        if let (Ok(metric_id), Ok(settings)) = futures::join!(
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
            services::settings::get(&pool)
        ) {
            let mut token: Option<String> = None;
            if let Some(id) = metric_id {
                if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                    token = Some(metric_token.to_string());
                }
            }
            
            #[derive(Debug)]
            struct Illustration {
                path: String,
                name: Option<String>,
                // fallback_path: String
            }
            #[derive(Template)]
            #[template(path = "components/project_tile.html")]
            struct ProjectTile {
                name: String,
                uri: String,
                illustration: Illustration,
                fallback_illustration: Illustration,
                categories: Vec<services::projects::Category>,
            }
            #[derive(Template)]
            #[template(path = "pages/portfolio.html")]
            struct Portfolio {
                categories: Vec<services::projects::Category>,
                projects: Vec<ProjectTile>,
                title: String,
                description: Option<String>,
                year: i32,
                metric_token: Option<String>,
                settings: services::settings::Settings
            }
            let (_, projects, categories) = futures::join!(
                metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)),
                services::projects::get_all(&pool, None),
                services::projects::categories::get_all(&pool, None)
            );
            let mut formatted_projects = vec![];
    
            for project in &projects {
                let project_categories = sqlx::query!(
                    "SELECT category_id FROM projects_categories WHERE project_id = $1",
                    project.id
                )
                .fetch_all(pool.as_ref())
                .await
                .unwrap();
                let mut c = vec![];
    
                for project_category in project_categories {
                    if let Some(category) = categories
                        .iter()
                        .find(|category| category.id == project_category.category_id)
                    {
                        c.push(category.clone());
                    }
                }
    
                let illustration = match sqlx::query_as!(
                    Illustration,
                    r#"SELECT
                            f.path AS "path", f.name AS "name"
                        FROM project_assets pa
                        JOIN files f ON f.id = pa.file_id
                        WHERE pa.project_id = $1 AND pa.order = 0"#,
                    project.id
                )
                .fetch_one(pool.as_ref())
                .await
                {
                    Ok(illustration) => illustration,
                    Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
                };
    
                formatted_projects.push(ProjectTile {
                    name: project.name.clone(),
                    uri: slugify(&format!("{}-{}", project.name, project.id)),
                    fallback_illustration: Illustration {
                        path: format!(
                            "{}.webp",
                            illustration
                                .path
                                .clone()
                                .split('.')
                                .collect::<Vec<_>>()
                                .get(0)
                                .unwrap()
                        ),
                        name: None,
                    },
                    illustration,
                    categories: c,
                });
            }
    
            return Portfolio {
                categories,
                projects: formatted_projects,
                title: page.title,
                description: page.description,
                year: chrono::Utc::now().year(),
                metric_token: token,
                settings
            }
            .into_response();
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/{name}-{id}")]
async fn view_project(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    web::Path((_, id)): web::Path<(String, i16)>,
) -> Result<HttpResponse, Error> {
    if !services::projects::exists(&pool, id).await {
        return Ok(HttpResponse::NotFound().finish())
    }

    if let (Ok(metric_id), Ok(project), assets, Ok(settings)) = futures::join!(
        metrics::add(&pool, &req, services::metrics::BelongsTo::Project(id)),
        services::projects::get(&pool, id),
        services::projects::assets::get_all(&pool, id),
        services::settings::get(&pool)
    ) {
        let mut token: Option<String> = None;
        if let Some(id) = metric_id {
            if let Ok(metric_token) = services::metrics::tokens::add(&pool, id).await {
                token = Some(metric_token.to_string());
            }
        }

        #[derive(Template)]
        #[template(path = "pages/portfolio_project.html")]
        struct PortfolioProject<'a> {
            title: String,
            description: Option<String>,
            content: String,
            date: String,
            international_date: String,
            asset_0: Option<&'a services::projects::Asset>,
            asset_1: Option<&'a services::projects::Asset>,
            assets: Option<Vec<services::projects::Asset>>,
            year: i32,
            metric_token: Option<String>,
            settings: services::settings::Settings
        }
    
        return PortfolioProject {
            title: project.name,
            description: project.description,
            content: project.content,
            date: project.date.format("%d/%m/%Y").to_string(),
            international_date: project.date.to_rfc3339(),
            asset_0: assets.get(0),
            asset_1: assets.get(1),
            assets: if assets.len() > 2 && assets.len() - 2 > 0 {
                Some(assets.get(2..).unwrap().to_vec())
            } else {
                None
            },
            year: chrono::Utc::now().year(),
            metric_token: token,
            settings
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[cfg(test)]
mod tests {
    use crate::create_pool;
    use actix_web::{test, web, App};
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_index() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/portfolio").service(super::index)),
        )
        .await;
        let resp = test::TestRequest::get()
            .uri("/portfolio")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    // #[actix_rt::test]
    // async fn test_project() {
    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();
    //     let mut app = test::init_service(
    //         App::new()
    //             .data(pool.clone())
    //             .service(web::scope("/portfolio").service(super::get_project)),
    //     )
    //     .await;
    //     let resp = test::TestRequest::get()
    //         .uri("/portfolio/lorem-1")
    //         .send_request(&mut app)
    //         .await;

    //     assert!(resp.status().is_success());
    // }
}
