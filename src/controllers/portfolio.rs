use super::metrics;
use crate::{services, utils::patch::Patch};
use actix_extract_multipart::*;
use actix_identity::Identity;
use actix_web::{delete, get, patch, post, put, web, Error, HttpRequest, HttpResponse};
use ammonia::Builder;
use askama_actix::{Template, TemplateIntoResponse};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::HashSet, ops::DerefMut, path::Path};

#[get("")]
async fn index(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    if let Ok(page) = services::pages::get(&pool, "portfolio").await {
        use slugmin::slugify;

        let mut token: Option<String> = None;
        if let Ok(Some(id)) =
            metrics::add(&pool, &req, services::metrics::BelongsTo::Page(page.id)).await
        {
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

            let illustration = sqlx::query_as!(
                Illustration,
                r#"SELECT
                        f.path AS "path", f.name AS "name"
                    FROM project_assets pa
                    JOIN files f ON f.id = pa.file_id
                    WHERE pa.project_id = $1 AND pa.order = 1"#,
                project.id
            )
            .fetch_one(pool.as_ref())
            .await
            .unwrap();

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
        }
        .into_response();
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[get("/{name}-{id}")]
async fn view_project(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    web::Path((_, id)): web::Path<(String, i16)>,
) -> Result<HttpResponse, Error> {
    if services::projects::exists(&pool, id).await {
        let (project, assets) = futures::join!(
            services::projects::get(&pool, id),
            services::projects::assets::get_all(&pool, id)
        );

        if let Ok(project) = project {
            let mut token: Option<String> = None;

            if let Ok(Some(id)) =
                metrics::add(&pool, &req, services::metrics::BelongsTo::Project(id)).await
            {
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
            }
            .into_response();
        }
    }

    Ok(HttpResponse::NotFound().finish())
}

#[get("/projects/{id}")]
pub async fn get_project(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::projects::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    #[derive(sqlx::FromRow, Serialize)]
    struct Project {
        name: String,
        description: Option<String>,
        content: String,
    }

    #[derive(Serialize)]
    struct Asset {
        id: i16,
        path: String,
        order: usize
    }

    let (project, assets, categories) = futures::join!(
        services::projects::get_spe::<Project>(&pool, "name, description, content", id),
        services::projects::assets::get_all(&pool, id),
        services::projects::categories::get_all(&pool, Some(id))
    );
    let categories = categories
        .iter()
        .map(|category| category.id)
        .collect::<Vec<i16>>();

    match project {
        Ok(project) => {
            HttpResponse::Ok().json(serde_json::json!({
                "title": project.name,
                "description": project.description,
                "content": project.content,
                // "categories":
                "assets": assets
                    .iter()
                    .enumerate()
                    .map(|(i, asset)| Asset {
                        id: asset.id,
                        path: asset.path.clone(),
                        order: i
                    })
                    .collect::<Vec<Asset>>(),
                "categories": categories
            }))
        }
        Err(e) => {
            eprintln!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct CategoryForm {
    name: String,
}

impl CategoryForm {
    fn is_valid(&mut self) -> bool {
        self.name = self.name.trim().to_string();

        self.name.len() <= 30
    }
}

#[post("/categories")]
async fn create_category(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: web::Form<CategoryForm>,
) -> HttpResponse {
    if session.identity().is_some() {
        if !form.is_valid() {
            return HttpResponse::BadRequest().finish();
        }

        return match services::projects::categories::insert(&pool, &form.name).await {
            Ok(id) => HttpResponse::Created().json(id),
            _ => HttpResponse::InternalServerError().finish(),
        };
    }

    HttpResponse::Unauthorized().finish()
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UpdateCategoryForm {
    #[serde(default)]
    name: Patch<String>,
    #[serde(default)]
    order: Patch<i16>,
}

impl UpdateCategoryForm {
    fn is_valid(&mut self) -> bool {
        if let Patch::Value(value) = &self.name {
            let value = value.trim().to_string();

            if value.len() > 30 {
                return false;
            }
        }

        if let Patch::Value(nb) = self.order {
            if nb < 0 {
                return false;
            }
        }

        true
    }
}

#[put("/categories/{id}")]
async fn update_category(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: web::Form<UpdateCategoryForm>,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_some() {
        if !services::projects::categories::exists(&pool, id).await {
            return HttpResponse::NotFound().finish();
        }

        if !form.is_valid() {
            return HttpResponse::BadRequest().finish();
        }

        return match services::projects::categories::update_2(
            &pool,
            id,
            crate::utils::patch::extract_fields(&*form),
        )
        .await
        {
            Ok(_) => return HttpResponse::Ok().finish(),
            Err(e) => {
                eprintln!("{:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        };
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/categories/{id}")]
async fn delete_category(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_some() {
        if services::projects::categories::exists(&pool, id).await {
            services::projects::categories::delete(&pool, id).await;

            return HttpResponse::Ok().finish();
        }

        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Unauthorized().finish()
}

#[derive(Deserialize, Debug)]
struct ProjectInformations {
    name: String,
    description: Option<String>,
    content: String,
    categories: Option<Vec<i16>>,
}

impl ProjectInformations {
    fn is_valid(&mut self) -> bool {
        self.name = self.name.trim().to_string();
        if let Some(description) = &mut self.description {
            *description = description.trim().to_string();
        }

        // Sanitize content for only print allowed html tags
        let mut allowed_tags: HashSet<&str> = HashSet::new();
        allowed_tags.insert("b");
        allowed_tags.insert("h2");
        allowed_tags.insert("h3");
        allowed_tags.insert("ul");
        allowed_tags.insert("ol");
        allowed_tags.insert("li");
        allowed_tags.insert("a");
        allowed_tags.insert("p");
        allowed_tags.insert("br");
        self.content = Builder::default()
            .tags(allowed_tags)
            .clean(self.content.trim())
            .to_string();

        self.name.len() <= 120
            && self.content.len() >= 30
            && ((self.description.is_some() && self.description.as_ref().unwrap().len() <= 320)
                || self.description.is_none())
    }
}

#[derive(Deserialize, Debug)]
pub struct ProjectAddForm {
    #[serde(flatten)]
    infos: ProjectInformations,
    files: Vec<actix_extract_multipart::File>, // TODO : change to Option
}

impl ProjectAddForm {
    async fn is_valid(&mut self) -> bool {
        if !self.infos.is_valid() {
            return false;
        }

        for file in &self.files {
            if !&["image/png", "image/jpeg", "image/webp"].contains(&file.file_type().as_str())
                || file.len() >= 2000000
            {
                return false;
            }
        }

        true
    }
}

#[post("/projects")]
pub async fn insert_project(
    pool: web::Data<PgPool>,
    mut form: actix_extract_multipart::Multipart<ProjectAddForm>,
    session: Identity,
) -> HttpResponse {
    if session.identity().is_some() {
        if !form.is_valid().await {
            return HttpResponse::BadRequest().finish();
        }

        // Check if specified categories exist
        if let Some(categories) = &form.infos.categories {
            for category_id in categories {
                if !services::projects::categories::exists(&pool, *category_id).await {
                    return HttpResponse::NotFound().finish();
                }
            }
        }

        return match services::projects::insert(
            &pool,
            &form.infos.name,
            form.infos.description.as_deref(),
            &form.infos.content,
        )
        .await
        {
            Ok(id) => {
                // Categories
                if let Some(categories) = &form.infos.categories {
                    let mut categories_fut = vec![];

                    for category_id in categories {
                        categories_fut.push(services::projects::link_to_category(
                            pool.as_ref(),
                            id,
                            *category_id,
                        ));
                    }

                    futures::future::join_all(categories_fut).await;
                }

                // Handle assets
                let mut i = 0;
                for file in &form.files {
                    let name = {
                        use slugmin::slugify;

                        slugify(&format!(
                            "{}_{}",
                            file.name(),
                            chrono::Utc::now().timestamp()
                        ))
                    };
                    let image = match image::load_from_memory(file.data()) {
                        Ok(image) => image,
                        Err(_) => return HttpResponse::BadRequest().finish(),
                    };

                    if crate::utils::image::create_images(
                        // file.data(),
                        &image,
                        &name,
                        Some((500, 500)),
                        Some((700, 700)),
                    )
                    .is_err()
                    {
                        return HttpResponse::BadRequest().finish();
                    }

                    let file_id = services::files::insert(
                        pool.get_ref(),
                        Some(file.name()),
                        &format!(
                            "{}.{}",
                            &name.clone(),
                            if image.color().has_alpha() {
                                "png"
                            } else {
                                "jpg"
                            }
                        ),
                    )
                    .await
                    .unwrap();
                    services::projects::assets::insert(pool.get_ref(), id, file_id, i)
                        .await
                        .unwrap();

                    i += 1;
                }

                HttpResponse::Created().json(id)
            }
            _ => HttpResponse::InternalServerError().finish(),
        };
    }

    HttpResponse::Unauthorized().finish()
}

#[derive(Deserialize, Debug)]
pub struct ProjectUpdateAssetForm {
    id: i16,
    order: Option<i16>,
    //     file: Option<File>,
    to_delete: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectUpdateForm {
    #[serde(default)]
    name: Patch<String>,
    // #[serde(default)]
    description: Patch<Option<String>>,
    #[serde(default)]
    content: Patch<String>,
    #[serde(default)]
    categories: Patch<Option<Vec<i16>>>,
    #[serde(default, skip_serializing)]
    // assets: Patch<Vec<ProjectUpdateAssetForm>>,
    assets: Patch<Vec<String>>,
    #[serde(skip_serializing)]
    files: Option<Vec<actix_extract_multipart::File>>,
}

#[patch("/projects/{id}")]
pub async fn update_project(
    pool: web::Data<PgPool>,
    mut form: actix_extract_multipart::Multipart<ProjectUpdateForm>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::projects::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    match &form.name {
        Patch::Null => return HttpResponse::BadRequest().finish(),
        Patch::Value(name) => {
            let name = name.trim().to_string();

            if name.is_empty() || name.len() > 120 {
                return HttpResponse::BadRequest().finish();
            }

            form.name = Patch::Value(name);
        }
        _ => (),
    }

    if let Patch::Value(Some(description)) = &form.description {
        let description = description.trim().to_string();

        if description.len() > 320 {
            return HttpResponse::BadRequest().finish();
        }

        form.description = Patch::Value(Some(description));
    }

    let mut transaction = pool.begin().await.unwrap();
    let mut images_to_delete = vec![];

    services::projects::detach_categories(transaction.deref_mut(), id).await;
    if let Patch::Value(categories) = &form.categories {
        if let Some(categories) = &categories {
            for category_id in categories {
                if !services::projects::categories::exists(&pool, *category_id).await {
                    return HttpResponse::NotFound().finish();
                } else {
                    if services::projects::link_to_category(
                        transaction.deref_mut(),
                        id,
                        *category_id,
                    )
                    .await
                    .is_err()
                    {
                        return HttpResponse::InternalServerError().finish();
                    }
                }
            }
        }
    }

    if let Patch::Value(content) = &form.content {
        let mut allowed_tags: HashSet<&str> = HashSet::new();
        allowed_tags.insert("b");
        allowed_tags.insert("h2");
        allowed_tags.insert("h3");
        allowed_tags.insert("ul");
        allowed_tags.insert("ol");
        allowed_tags.insert("li");
        allowed_tags.insert("a");
        allowed_tags.insert("p");
        allowed_tags.insert("br");

        let content = Builder::default()
            .tags(allowed_tags)
            .clean(content.trim())
            .to_string();

        if content.is_empty() || content.len() > 1000 {
            return HttpResponse::BadRequest().finish();
        }

        form.content = Patch::Value(content);
    }

    if let Patch::Value(assets) = &form.assets {
        for asset in assets {
            match serde_json::from_str::<ProjectUpdateAssetForm>(asset) {
                Ok(asset) => {
                    if let Some(true) = asset.to_delete {
                        #[derive(sqlx::FromRow)]
                        struct Asset {
                            path: String
                        }

                        match services::projects::assets::get::<Asset>(&pool, "path", asset.id).await {
                            Ok(asset) => {
                                let filename = asset.path.split('.').collect::<Vec<_>>();
                                let filename = filename.get(0).unwrap();

                                images_to_delete.append(&mut [
                                    format!("./uploads/mobile/{}", asset.path),
                                    format!("./uploads/mobile/{}.webp", filename),
                                    format!("./uploads/{}", asset.path),
                                    format!("./uploads/{}.webp", filename),
                                ].to_vec());
                            },
                            Err(_) => return HttpResponse::InternalServerError().finish()
                        }

                        services::projects::assets::delete(transaction.deref_mut(), asset.id).await;
                    } else {
                        if let Some(order) = asset.order {
                            if let Err(_) = services::projects::assets::update(transaction.deref_mut(), asset.id, order).await {
                                return HttpResponse::InternalServerError().finish()
                            }
                        }
                    }
                },
                Err(_) => return HttpResponse::BadRequest().finish()
            }
        }
    }

    if let Some(files) = &form.files {
        if services::projects::assets::count(&pool, id).await >= 5 {
            // cant insert more assets
        } else {
            let mut available_slots = services::projects::assets::get_available_slots(transaction.deref_mut(), id).await;
            println!("Available slots : {:?}", available_slots);
            
            for file in files {
                let name = {
                    use slugmin::slugify;

                    slugify(&format!(
                        "{}_{}",
                        file.name(),
                        chrono::Utc::now().timestamp()
                    ))
                };
                let image = match image::load_from_memory(file.data()) {
                    Ok(image) => image,
                    Err(_) => return HttpResponse::BadRequest().finish(),
                };

                if crate::utils::image::create_images(
                    &image,
                    &name,
                    Some((500, 500)),
                    Some((700, 700)),
                )
                .is_err()
                {
                    return HttpResponse::BadRequest().finish();
                }

                let file_id = services::files::insert(
                    pool.get_ref(),
                    Some(file.name()),
                    &format!(
                        "{}.{}",
                        &name.clone(),
                        if image.color().has_alpha() {
                            "png"
                        } else {
                            "jpg"
                        }
                    ),
                )
                .await
                .unwrap();
                services::projects::assets::insert(pool.get_ref(), id, file_id, available_slots[0])
                    .await
                    .unwrap();

                available_slots.remove(0);
            }
        }
    }

    let mut fields_need_update = crate::utils::patch::extract_fields(&*form);

    fields_need_update.remove("categories");

    // if services::projects::partial_update(transaction.deref_mut(), id, fields_need_update).await.is_err() {
    //     return HttpResponse::InternalServerError().finish()
    // }

    match services::projects::partial_update(transaction.deref_mut(), id, fields_need_update).await
    {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }

    transaction.commit().await.unwrap();

    println!("Remove files {:?}", images_to_delete);
    crate::utils::image::remove_files(&images_to_delete);

    HttpResponse::Ok().finish()
}

#[delete("/projects/{id}")]
async fn delete_project(
    pool: web::Data<PgPool>,
    web::Path(id): web::Path<i16>,
    session: Identity,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish()
    }

    if !services::projects::exists(&pool, id).await {
        return HttpResponse::NotFound().finish()
    }

    let assets = services::projects::assets::get_all(&pool, id).await;
    let mut files_to_delete = vec![];

    assets
        .iter()
        .for_each(|asset| {
            let filename = asset.path.split('.').collect::<Vec<_>>();
            let filename = filename.get(0).unwrap();

            files_to_delete.append(
                &mut [
                    format!("./uploads/mobile/{}", asset.path),
                    format!("./uploads/mobile/{}.webp", filename),
                    format!("./uploads/{}", asset.path),
                    format!("./uploads/{}.webp", filename),
                ].to_vec()
            );
        });

    services::projects::delete(&pool, id).await;
    crate::utils::image::remove_files(&files_to_delete);

    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct AssetForm {
    // TODO : remplace with video
    file: File,
    name: Option<String>,
    order: i16,
    is_visible: bool,
}

impl AssetForm {
    fn sanitize(&mut self) {
        if let Some(name) = &mut self.name {
            *name = name.trim().to_string();
        }
    }

    fn is_valid(&mut self) -> bool {
        self.sanitize();

        self.order > 0
    }
}

#[derive(Deserialize)]
pub struct UpdateAssetForm {
    name: Option<String>,
    order: i16,
}

#[patch("/projects/{project_id}/assets/{asset_id}")]
async fn update_asset(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path((project_id, asset_id)): web::Path<(i16, i16)>,
    form: web::Form<UpdateAssetForm>,
) -> HttpResponse {
    if session.identity().is_some() {
        if services::projects::assets::exists(&pool, project_id, asset_id).await {
            match sqlx::query!(
                "CALL update_asset($1, $2, $3)",
                asset_id,
                form.order,
                form.name
            )
            .execute(pool.as_ref())
            .await
            {
                Ok(_) => return HttpResponse::Ok().finish(),
                _ => return HttpResponse::InternalServerError().finish(),
            }
        }

        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/projects/{project_id}/assets/{asset_id}")]
async fn delete_asset(
    pool: web::Data<PgPool>,
    web::Path((project_id, asset_id)): web::Path<(i16, i16)>,
    session: Identity,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish()
    }

    if !services::projects::assets::exists(&pool, project_id, asset_id).await {
        return HttpResponse::NotFound().finish();
    }

    #[derive(sqlx::FromRow)]
    struct Asset {
        path: String
    }

    match services::projects::assets::get::<Asset>(&pool, "f.path", asset_id).await {
        Ok(asset) => {
            // TODO : remove all differents file formats
        
            let path = format!("./uploads/{}", asset.path);
            let path = Path::new(&path);
        
            if path.exists() {
                std::fs::remove_file(path).unwrap();
            }
        
            return HttpResponse::Ok().finish();

        },
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::create_pool;
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{cookie::Cookie, test, web, App};
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

    #[actix_rt::test]
    async fn test_project() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .data(pool.clone())
                .service(web::scope("/portfolio").service(super::get_project)),
        )
        .await;
        let resp = test::TestRequest::get()
            .uri("/portfolio/lorem-1")
            .send_request(&mut app)
            .await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_create_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(web::scope("/portfolio").service(super::create_category)),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 1
            }))
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_create_category_not_logged() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(web::scope("/portfolio").service(super::create_category)),
        )
        .await;

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 1
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 401);
    }

    #[actix_rt::test]
    async fn test_update_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::update_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 2
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::put()
            .uri(&format!("/portfolio/categories/{}", id))
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Dolor sit amet",
                "order": 2
            }))
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_update_category_not_logged() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::update_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 40
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::put()
            .uri(&format!("/portfolio/categories/{}", id))
            .set_form(&serde_json::json!({
                "name": "Dolor sit amet",
                "order": 50
            }))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 401);
    }

    #[actix_rt::test]
    async fn test_update_category_doesnt_exist() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::update_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());
        assert!(res.status().is_success());

        let res = test::TestRequest::put()
            .uri("/portfolio/categories/999")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Dolor sit amet",
                "order": 2
            }))
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_category() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::delete_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());
        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 3
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::delete()
            .uri(&format!("/portfolio/categories/{}", id))
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .send_request(&mut app)
            .await;

        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_category_not_logged() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::delete_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        assert!(cookie.is_some());

        assert!(res.status().is_success());

        let res = test::TestRequest::post()
            .uri("/portfolio/categories")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .set_form(&serde_json::json!({
                "name": "Lorem ipsum",
                "order": 10
            }))
            .send_request(&mut app)
            .await;

        let id: i16 = test::read_body_json(res).await;

        let res = test::TestRequest::delete()
            .uri(&format!("/portfolio/categories/{}", id))
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 401);
    }

    #[actix_rt::test]
    async fn test_delete_category_doesnt_exist() {
        dotenv().ok();

        let pool = create_pool().await.unwrap();
        let mut app = test::init_service(
            App::new()
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .secure(true),
                ))
                .data(pool.clone())
                .service(web::scope("/user").service(crate::controllers::user::login))
                .service(
                    web::scope("/portfolio")
                        .service(super::create_category)
                        .service(super::delete_category),
                ),
        )
        .await;
        let res = test::TestRequest::post()
            .uri("/user/login")
            .set_form(&serde_json::json!({
                "email": "contact@ludivinefarat.fr",
                "password": "root"
            }))
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());

        let cookie = res.headers().get(actix_web::http::header::SET_COOKIE);

        let res = test::TestRequest::delete()
            .uri("/portfolio/categories/9999")
            .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
            .send_request(&mut app)
            .await;

        assert_eq!(res.status(), 404);
    }

    // TODO : implement delete projects tests
    #[actix_rt::test]
    async fn test_delete_project() {
        //     dotenv().ok();

        //     let pool = create_pool().await.unwrap();
        //     let mut app =
        //         test::init_service(App::new().data(pool.clone()).service(super::get_project)).await;
        //     let resp = test::TestRequest::get()
        //         .uri("/portfolio/lorem-1")
        //         .send_request(&mut app)
        //         .await;

        //     assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_project_doesnt_exists() {}

    #[actix_rt::test]
    async fn test_delete_project_not_logged() {}
}
