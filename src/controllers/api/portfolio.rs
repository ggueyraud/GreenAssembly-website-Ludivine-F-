use crate::{services, utils::patch::Patch};
use actix_identity::Identity;
use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use ammonia::Builder;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::HashSet, ops::DerefMut};

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
        order: usize,
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
        Err(_) => HttpResponse::InternalServerError().finish(),
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

        return match services::projects::categories::partial_update(
            &pool,
            id,
            crate::utils::patch::extract_fields(&*form),
        )
        .await
        {
            Ok(_) => return HttpResponse::Ok().finish(),
            Err(_) => HttpResponse::InternalServerError().finish(),
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
pub struct ProjectAddForm {
    name: String,
    description: Option<String>,
    content: String,
    categories: Option<Vec<i16>>,
    files: Vec<actix_extract_multipart::File>, // TODO : change to Option
}

#[post("/projects")]
pub async fn insert_project(
    pool: web::Data<PgPool>,
    mut form: actix_extract_multipart::Multipart<ProjectAddForm>,
    session: Identity,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    form.name = form.name.trim().to_string();

    if form.name.len() > 120 {
        return HttpResponse::BadRequest().finish();
    }

    if let Some(description) = &mut form.description {
        *description = description.trim().to_string();

        if description.len() > 320 {
            return HttpResponse::BadRequest().finish();
        }
    }

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

    form.content = Builder::default()
        .tags(allowed_tags)
        .clean(form.content.trim())
        .to_string();

    if form.content.len() < 30 {
        return HttpResponse::BadRequest().finish();
    }

    // Check if specified categories exist
    if let Some(categories) = &form.categories {
        for category_id in categories {
            if !services::projects::categories::exists(&pool, *category_id).await {
                return HttpResponse::NotFound().finish();
            }
        }
    }

    let mut uploader = crate::utils::image::Uploader::new();
    let mut transaction = pool.begin().await.unwrap();

    return match services::projects::insert(
        transaction.deref_mut(),
        &form.name,
        form.description.as_deref(),
        &form.content,
    )
    .await
    {
        Ok(id) => {
            // Categories
            if let Some(categories) = &form.categories {
                for category_id in categories {
                    if services::projects::link_to_category(
                        transaction.deref_mut(),
                        id,
                        *category_id,
                    )
                    .await
                    .is_err()
                    {
                        return HttpResponse::InternalServerError().finish();
                    };
                }
            }

            // Handle assets
            for (i, file) in form.files.iter().enumerate() {
                let name = {
                    use slugmin::slugify;

                    slugify(&format!(
                        "{}_{}",
                        file.name(),
                        chrono::Utc::now().timestamp()
                    ))
                };
                match image::load_from_memory(file.data()) {
                    Ok(image) => {
                        if uploader
                            .handle(&image, &name, Some((500, 500)), Some((700, 700)))
                            .is_err()
                        {
                            return HttpResponse::InternalServerError().finish();
                        }

                        match services::files::insert(
                            transaction.deref_mut(),
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
                        {
                            Ok(file_id) => {
                                if services::projects::assets::insert(
                                    transaction.deref_mut(),
                                    id,
                                    file_id,
                                    i as i16,
                                )
                                .await
                                .is_err()
                                {
                                    return HttpResponse::InternalServerError().finish();
                                }
                            }
                            Err(_) => return HttpResponse::InternalServerError().finish(),
                        }
                    }
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                }
            }

            uploader.clear();
            HttpResponse::Created().json(id)
        }
        _ => HttpResponse::InternalServerError().finish(),
    };
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

    let mut uploader = crate::utils::image::Uploader::new();
    let mut transaction = pool.begin().await.unwrap();
    let mut images_to_delete = vec![];

    services::projects::detach_categories(transaction.deref_mut(), id).await;

    if let Patch::Value(Some(categories)) = &form.categories {
        for category_id in categories {
            if !services::projects::categories::exists(&pool, *category_id).await {
                return HttpResponse::NotFound().finish();
            } else if services::projects::link_to_category(
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
                            path: String,
                        }

                        match services::projects::assets::get::<Asset>(&pool, "path", asset.id)
                            .await
                        {
                            Ok(asset) => {
                                let filename = asset.path.split('.').collect::<Vec<_>>();
                                let filename = filename.get(0).unwrap();

                                images_to_delete.append(
                                    &mut [
                                        format!("./uploads/mobile/{}", asset.path),
                                        format!("./uploads/mobile/{}.webp", filename),
                                        format!("./uploads/{}", asset.path),
                                        format!("./uploads/{}.webp", filename),
                                    ]
                                    .to_vec(),
                                );
                            }
                            Err(_) => return HttpResponse::InternalServerError().finish(),
                        }

                        services::projects::assets::delete(transaction.deref_mut(), asset.id).await;
                    } else if let Some(order) = asset.order {
                        if services::projects::assets::update(
                            transaction.deref_mut(),
                            asset.id,
                            order,
                        )
                        .await
                        .is_err()
                        {
                            return HttpResponse::InternalServerError().finish();
                        }
                    }
                }
                Err(_) => return HttpResponse::BadRequest().finish(),
            }
        }
    }

    if let Some(files) = &form.files {
        if services::projects::assets::count(&pool, id).await >= 5 {
            // cant insert more assets
        } else {
            let mut available_slots =
                services::projects::assets::get_available_slots(transaction.deref_mut(), id).await;

            for file in files {
                let name = {
                    use slugmin::slugify;

                    slugify(&format!(
                        "{}_{}",
                        file.name(),
                        chrono::Utc::now().timestamp()
                    ))
                };

                match image::load_from_memory(file.data()) {
                    Ok(image) => {
                        if uploader
                            .handle(&image, &name, Some((500, 500)), Some((700, 700)))
                            .is_err()
                        {
                            return HttpResponse::InternalServerError().finish();
                        }

                        match services::files::insert(
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
                        {
                            Ok(file_id) => {
                                if services::projects::assets::insert(
                                    pool.get_ref(),
                                    id,
                                    file_id,
                                    available_slots[0],
                                )
                                .await
                                .is_err()
                                {
                                    return HttpResponse::InternalServerError().finish();
                                }

                                available_slots.remove(0);
                            }
                            Err(_) => return HttpResponse::InternalServerError().finish(),
                        }
                    }
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                }
            }
        }
    }

    let mut fields_need_update = crate::utils::patch::extract_fields(&*form);

    fields_need_update.remove("categories");

    if services::projects::partial_update(transaction.deref_mut(), id, fields_need_update)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    transaction.commit().await.unwrap();
    uploader.clear();
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
        return HttpResponse::Unauthorized().finish();
    }

    if !services::projects::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    let assets = services::projects::assets::get_all(&pool, id).await;
    let mut files_to_delete = vec![];

    assets.iter().for_each(|asset| {
        let filename = asset.path.split('.').collect::<Vec<_>>();
        let filename = filename.get(0).unwrap();

        files_to_delete.append(
            &mut [
                format!("./uploads/mobile/{}", asset.path),
                format!("./uploads/mobile/{}.webp", filename),
                format!("./uploads/{}", asset.path),
                format!("./uploads/{}.webp", filename),
            ]
            .to_vec(),
        );
    });

    services::projects::delete(&pool, id).await;
    crate::utils::image::remove_files(&files_to_delete);

    HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::create_pool;
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{cookie::Cookie, test, web, App};
    use dotenv::dotenv;

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
