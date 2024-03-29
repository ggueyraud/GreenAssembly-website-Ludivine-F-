use crate::{services, utils::image::Uploader, utils::patch::Patch};
use actix_identity::Identity;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slugmin::slugify;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::ops::DerefMut;

#[get("/categories/{id}")]
async fn get_category(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::blog::categories::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    #[derive(FromRow)]
    struct Category {
        name: String,
        description: Option<String>,
        is_visible: Option<bool>,
        is_seo: Option<bool>,
    }

    match services::blog::categories::get::<Category>(
        &pool,
        "name, description, is_visible, is_seo",
        id,
    )
    .await
    {
        Ok(category) => HttpResponse::Ok().json(serde_json::json!({
            "id": id,
            "name": category.name,
            "description": category.description,
            "is_visible": category.is_visible,
            "is_seo": category.is_seo
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize)]
pub struct NewCategoryForm {
    name: String,
    description: Option<String>,
    is_visible: Option<bool>,
    is_seo: Option<bool>,
}

#[post("/categories")]
async fn insert_category(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: web::Json<NewCategoryForm>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    form.name = form.name.trim().to_string();
    form.description = form
        .description
        .as_ref()
        .as_ref()
        .map(|description| description.trim().to_string());

    if let Ok(id) = services::blog::categories::insert(
        &pool,
        &form.name,
        form.description.as_deref(),
        form.is_visible,
        form.is_seo,
    )
    .await
    {
        if services::blog::categories::update_uri(
            &pool,
            id,
            &slugify(&format!("{}-{}", form.name, id)),
        )
        .await
        .is_ok()
        {
            return HttpResponse::Created().json(id);
        }
    }

    HttpResponse::InternalServerError().finish()
}

#[derive(Deserialize, Serialize)]
pub struct UpdateCategoryForm {
    #[serde(default)]
    name: Patch<String>,
    #[serde(default)]
    description: Patch<Option<String>>,
    #[serde(default)]
    is_visible: Patch<Option<bool>>,
    #[serde(default)]
    is_seo: Patch<Option<bool>>,
    #[serde(default)]
    order: Patch<i16>,
}

#[patch("/categories/{id}")]
async fn update_category(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
    mut form: web::Json<UpdateCategoryForm>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::blog::categories::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    match &form.name {
        Patch::Null => return HttpResponse::BadRequest().finish(),
        Patch::Value(name) => {
            let name = name.trim().to_string();

            if name.is_empty() || name.len() > 60 {
                return HttpResponse::BadRequest().finish();
            }

            form.name = Patch::Value(name);
        }
        _ => (),
    }

    if let Patch::Value(Some(description)) = &form.description {
        // sanitize html content
        let mut allowed_tags = std::collections::HashSet::<&str>::new();
        allowed_tags.insert("b");
        let description = ammonia::Builder::default()
            .tags(allowed_tags)
            .clean(description.trim())
            .to_string();

        if description.len() > 320 {
            return HttpResponse::BadRequest().finish();
        }

        form.description = Patch::Value(Some(description));
    }

    let mut fields_to_update = crate::utils::patch::extract_fields(&*form);

    if let Patch::Value(name) = &form.name {
        fields_to_update.insert(
            String::from("uri"),
            serde_json::json!(slugify(&format!("{}-{}", name, id))),
        );
    }

    if services::blog::categories::partial_update(pool.get_ref(), id, fields_to_update)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[delete("/categories/{id}")]
async fn delete_category(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::blog::categories::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    services::blog::categories::delete(&pool, id).await;

    return HttpResponse::Ok().finish();
}

#[get("/articles/{id}")]
async fn get_article(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::blog::articles::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    #[derive(FromRow, Serialize)]
    struct Article {
        cover: String,
        title: String,
        description: Option<String>,
        content: String,
        is_published: Option<bool>,
        is_seo: Option<bool>,
    }

    #[derive(Serialize)]
    struct File {
        id: String,
        path: String,
    }

    let (article, images) = futures::join!(
        services::blog::articles::get::<Article>(
            &pool,
            r#"f.path AS "cover", title, description, content, is_published, is_seo"#,
            id
        ),
        services::blog::articles::images::get_all(&pool, id)
    );

    if let Ok(article) = article {
        return HttpResponse::Ok().json(serde_json::json!({
            "id": id,
            "cover": article.cover,
            "title": article.title,
            "description": article.description,
            "content": article.content,
            "is_published": article.is_published,
            "is_seo": article.is_seo,
            "images": serde_json::json!(
                images
                    .iter()
                    .map(|image| File {
                        id: image.id.to_string(),
                        path: image.path.clone()
                    })
                    .collect::<Vec<File>>()
            )
        }));
    }

    HttpResponse::InternalServerError().finish()
}

#[derive(Deserialize)]
pub struct NewArticleForm {
    cover: actix_extract_multipart::File,
    category_id: Option<i16>,
    title: String,
    description: Option<String>,
    is_published: Option<bool>,
    is_seo: Option<bool>,
    content: String,
    pictures: Option<Vec<actix_extract_multipart::File>>,
}

#[post("/articles")]
async fn insert_article(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: actix_extract_multipart::Multipart<NewArticleForm>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    form.title = form.title.trim().to_string();
    let mut uploader = Uploader::new();

    if form.title.is_empty() || form.title.len() > 255 {
        return HttpResponse::BadRequest().finish();
    }
    if let Some(description) = form.description.clone() {
        let description = description.trim().to_string();

        if description.len() > 320 {
            return HttpResponse::BadRequest().finish();
        }

        form.description = Some(description);
    }
    if let Some(category_id) = form.category_id {
        if !services::blog::categories::exists(&pool, category_id).await {
            return HttpResponse::NotFound().finish();
        }
    }

    let mut transaction = pool.begin().await.unwrap();
    let cover_id = match image::load_from_memory(form.cover.data()) {
        Ok(image) => {
            let name = format!("cover_{}", chrono::Utc::now().timestamp());

            if uploader
                .handle(&image, &name, Some((500, 250)), Some((700, 350)), true)
                .is_err()
            {
                return HttpResponse::InternalServerError().finish();
            }

            let file_id = match services::files::insert(
                transaction.deref_mut(),
                None,
                &format!(
                    "{}.{}",
                    name,
                    if image.color().has_alpha() {
                        "png"
                    } else {
                        "jpg"
                    }
                ),
            )
            .await
            {
                Ok(file_id) => file_id,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

            file_id
        }
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let mut allowed_tags = std::collections::HashSet::<&str>::new();
    allowed_tags.insert("b");
    allowed_tags.insert("ul");
    allowed_tags.insert("ol");
    allowed_tags.insert("li");
    allowed_tags.insert("a");
    allowed_tags.insert("p");
    allowed_tags.insert("br");

    form.content = ammonia::Builder::default()
        .tags(allowed_tags)
        .clean(form.content.trim())
        .to_string();

    if let Ok(id) = services::blog::articles::insert(
        transaction.deref_mut(),
        form.category_id,
        cover_id,
        &form.title,
        form.description.as_deref(),
        &form.content,
        form.is_published,
        form.is_seo,
    )
    .await
    {
        let mut content = form.content.clone();

        if let Some(pictures) = &form.pictures {
            for (i, image) in pictures.iter().enumerate() {
                if !&["image/png", "image/jpeg"].contains(&image.file_type().as_str())
                    || image.len() > 2000000
                {
                    return HttpResponse::BadRequest().finish();
                }

                match image::load_from_memory(image.data()) {
                    Ok(image) => {
                        let name = format!("{}_{}_{}", id, i, chrono::Utc::now().timestamp());

                        if uploader
                            .handle(&image, &name, Some((500, 500)), Some((700, 700)), true)
                            .is_err()
                        {
                            return HttpResponse::InternalServerError().finish();
                        }

                        if let Ok(file_id) = services::files::insert(
                            transaction.deref_mut(),
                            None,
                            &format!(
                                "{}.{}",
                                name,
                                if image.color().has_alpha() {
                                    "png"
                                } else {
                                    "jpg"
                                }
                            ),
                        )
                        .await
                        {
                            match services::blog::articles::images::insert(
                                transaction.deref_mut(),
                                id,
                                file_id,
                            )
                            .await
                            {
                                Ok(id) => {
                                    content = content.replacen(
                                        &format!("[[{}]]", i),
                                        &format!("[[{}]]", id),
                                        1,
                                    );
                                }
                                Err(_) => return HttpResponse::InternalServerError().finish(),
                            }
                        } else {
                            return HttpResponse::InternalServerError().finish();
                        };
                    }
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                };
            }
        }

        let mut fields_to_update = HashMap::new();
        fields_to_update.insert(
            String::from("uri"),
            Value::String(slugify(&format!("{}-{}", form.title, id))),
        );
        fields_to_update.insert(String::from("content"), Value::String(content));

        if services::blog::articles::partial_update(transaction.deref_mut(), id, fields_to_update)
            .await
            .is_err()
        {
            return HttpResponse::InternalServerError().finish();
        }

        transaction.commit().await.unwrap();

        uploader.clear();

        return HttpResponse::Created().json(id);
    }

    HttpResponse::InternalServerError().finish()
}

#[derive(Deserialize, Serialize)]
pub struct UpdateArticleForm {
    #[serde(default, skip_serializing)]
    cover: Patch<actix_extract_multipart::File>,
    #[serde(default)]
    category_id: Patch<Option<i16>>,
    #[serde(default)]
    title: Patch<String>,
    #[serde(default)]
    description: Patch<Option<String>>,
    #[serde(default)]
    content: Patch<String>,
    #[serde(default)]
    is_published: Patch<bool>,
    #[serde(default)]
    is_seo: Patch<bool>,
    #[serde(default, skip_serializing)]
    pictures: Patch<Option<Vec<actix_extract_multipart::File>>>,
}

#[patch("/articles/{id}")]
async fn update_article(
    pool: web::Data<PgPool>,
    session: Identity,
    mut form: actix_extract_multipart::Multipart<UpdateArticleForm>,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::blog::articles::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    #[derive(FromRow)]
    struct Article {
        cover_id: i32,
    }

    let mut uploader = Uploader::new();

    let Article { cover_id } = if let Ok(article) =
        services::blog::articles::get::<Article>(&pool, "cover_id", id).await
    {
        article
    } else {
        return HttpResponse::NotFound().finish();
    };

    match &form.title {
        Patch::Null => return HttpResponse::BadRequest().finish(),
        Patch::Value(title) => {
            let title = title.trim().to_string();

            if title.is_empty() || title.len() > 255 {
                return HttpResponse::BadRequest().finish();
            }

            form.title = Patch::Value(title);
        }
        _ => (),
    }

    if let Patch::Value(Some(description)) = &form.description {
        // sanitize html content
        let mut allowed_tags = std::collections::HashSet::<&str>::new();
        allowed_tags.insert("b");
        let description = ammonia::Builder::default()
            .tags(allowed_tags)
            .clean(description.trim())
            .to_string();

        if description.is_empty() || description.len() > 320 {
            return HttpResponse::BadRequest().finish();
        }

        form.description = Patch::Value(Some(description));
    }

    let mut transaction = pool.begin().await.unwrap();
    let mut files_to_remove = vec![];

    if let Patch::Value(Some(category_id)) = form.category_id {
        if !services::blog::categories::exists(&pool, category_id).await {
            return HttpResponse::NotFound().finish();
        }
    }

    if let Patch::Value(content) = &form.content {
        let mut allowed_tags = std::collections::HashSet::<&str>::new();
        allowed_tags.insert("b");
        allowed_tags.insert("ul");
        allowed_tags.insert("ol");
        allowed_tags.insert("li");
        allowed_tags.insert("a");
        allowed_tags.insert("p");
        allowed_tags.insert("br");

        let images = services::blog::articles::images::get_all(pool.as_ref(), id).await;

        for image in &images {
            // If content doesnt has id anymore so the image has been deleted
            if !content.contains(&image.id.to_string()) {
                services::blog::articles::images::delete(transaction.deref_mut(), image.id).await;
                let filename = image.path.split('.').collect::<Vec<_>>();
                let filename = filename.get(0).unwrap();

                files_to_remove.append(
                    &mut [
                        format!("./uploads/mobile/{}", image.path),
                        format!("./uploads/mobile/{}.webp", filename),
                        format!("./uploads/{}", image.path),
                        format!("./uploads/{}.webp", filename),
                    ]
                    .to_vec(),
                );
            }
        }

        form.content = Patch::Value(
            ammonia::Builder::default()
                .tags(allowed_tags)
                .clean(content.trim())
                .to_string(),
        );
    }

    if let Patch::Value(Some(pictures)) = &form.pictures {
        let mut content = if let Patch::Value(content) = &form.content {
            content.clone()
        } else {
            return HttpResponse::BadRequest().finish();
        };

        for (i, image) in pictures.iter().enumerate() {
            if !&["image/png", "image/jpeg"].contains(&image.file_type().as_str())
                || image.len() > 2000000
            {
                return HttpResponse::BadRequest().finish();
            }

            match image::load_from_memory(image.data()) {
                Ok(image) => {
                    let name = format!("{}_{}_{}", id, i, chrono::Utc::now().timestamp());

                    if uploader
                        .handle(&image, &name, Some((500, 500)), Some((700, 700)), true)
                        .is_err()
                    {
                        return HttpResponse::InternalServerError().finish();
                    }

                    let file_id = if let Ok(id) = services::files::insert(
                        transaction.deref_mut(),
                        None,
                        &format!(
                            "{}.{}",
                            name,
                            if image.color().has_alpha() {
                                "png"
                            } else {
                                "jpg"
                            }
                        ),
                    )
                    .await
                    {
                        id
                    } else {
                        return HttpResponse::InternalServerError().finish();
                    };

                    match services::blog::articles::images::insert(
                        transaction.deref_mut(),
                        id,
                        file_id,
                    )
                    .await
                    {
                        Ok(id) => {
                            content =
                                content.replacen(&format!("[[{}]]", i), &format!("[[{}]]", id), 1);
                        }
                        Err(_) => return HttpResponse::InternalServerError().finish(),
                    }
                }
                Err(_) => {
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }

        form.content = Patch::Value(content);
    }

    let mut fields_need_update = crate::utils::patch::extract_fields(&*form);

    if let Patch::Value(cover) = &form.cover {
        match image::load_from_memory(cover.data()) {
            Ok(image) => {
                let name = format!("cover_{}", chrono::Utc::now().timestamp());

                if uploader
                    .handle(&image, &name, Some((500, 250)), Some((700, 350)), true)
                    .is_err()
                {
                    return HttpResponse::BadRequest().finish();
                }

                let file_id = services::files::insert(
                    transaction.deref_mut(),
                    None,
                    &format!(
                        "{}.{}",
                        name,
                        if image.color().has_alpha() {
                            "png"
                        } else {
                            "jpg"
                        }
                    ),
                )
                .await
                .unwrap();

                // Delete old cover from disk
                #[derive(FromRow)]
                struct Cover {
                    path: String,
                }

                let path = if let Ok(cover) =
                    services::files::get::<Cover>(&pool, cover_id, "path").await
                {
                    cover.path
                } else {
                    return HttpResponse::InternalServerError().finish();
                };

                let old_cover_name = path.split('.').collect::<Vec<_>>();
                let old_cover_name = old_cover_name.get(0).unwrap();

                files_to_remove.append(
                    &mut [
                        format!("./uploads/mobile/{}", path),
                        format!("./uploads/mobile/{}.webp", old_cover_name),
                        format!("./uploads/{}", path),
                        format!("./uploads/{}.webp", old_cover_name),
                    ]
                    .to_vec(),
                );

                // Delete cover form field of the fields to be update, set
                // cover_id instead with new file id
                fields_need_update.remove("cover");
                fields_need_update
                    .insert(String::from("cover_id"), serde_json::Value::from(file_id));
            }
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }

    fields_need_update.remove("blocks");

    if services::blog::articles::partial_update(transaction.deref_mut(), id, fields_need_update)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    // If an cover has been supplied, remove old
    if let Patch::Value(_) = &form.cover {
        services::files::delete(transaction.deref_mut(), cover_id).await;
    }

    transaction.commit().await.unwrap();

    crate::utils::image::remove_files(&files_to_remove);

    uploader.clear();

    HttpResponse::Ok().finish()
}

#[delete("/articles/{id}")]
async fn delete_article(
    pool: web::Data<PgPool>,
    session: Identity,
    web::Path(id): web::Path<i16>,
) -> HttpResponse {
    if session.identity().is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    if !services::blog::articles::exists(&pool, id).await {
        return HttpResponse::NotFound().finish();
    }

    #[derive(FromRow)]
    struct Article {
        cover_id: i32,
    }

    #[derive(FromRow)]
    struct File {
        path: String,
    }

    let (article, images) = futures::join!(
        services::blog::articles::get::<Article>(&pool, "cover_id", id),
        services::blog::articles::images::get_all(&pool, id)
    );
    let mut images_to_delete = vec![];

    if let Ok(article) = article {
        if let Ok(file) = services::files::get::<File>(&pool, article.cover_id, "path").await {
            let cover_file_name = file.path.split('.').collect::<Vec<_>>();
            let cover_file_name = cover_file_name.get(0).unwrap();

            images_to_delete.append(
                &mut [
                    format!("./uploads/mobile/{}", file.path),
                    format!("./uploads/mobile/{}.webp", cover_file_name),
                    format!("./uploads/{}", file.path),
                    format!("./uploads/{}.webp", cover_file_name),
                ]
                .to_vec(),
            );

            for image in &images {
                let file_name = image.path.split('.').collect::<Vec<_>>();
                let file_name = file_name.get(0).unwrap();

                images_to_delete.append(
                    &mut [
                        format!("./uploads/mobile/{}", image.path),
                        format!("./uploads/mobile/{}.webp", file_name),
                        format!("./uploads/{}", image.path),
                        format!("./uploads/{}.webp", file_name),
                    ]
                    .to_vec(),
                );
            }

            services::blog::articles::delete(&pool, id).await;

            crate::utils::image::remove_files(&images_to_delete);

            return HttpResponse::Ok().finish();
        }
    }

    HttpResponse::InternalServerError().finish()
}

#[cfg(test)]
mod tests {
    use crate::controllers;
    use crate::create_pool;
    use crate::CookieIdentityPolicy;
    use crate::IdentityService;
    use actix_web::http::header;
    use actix_web::{cookie::Cookie, http, http::StatusCode, test, web, App};
    use dotenv::dotenv;
    use std::str::FromStr;

    // #[actix_rt::test]
    // async fn test_insert_category() {
    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();
    //     let mut app = test::init_service(
    //         App::new()
    //             .wrap(IdentityService::new(
    //                 CookieIdentityPolicy::new(&[0; 32])
    //                     .name("auth-cookie")
    //                     .secure(true),
    //             ))
    //             .data(pool.clone())
    //             .service(web::scope("/api/blog").service(controllers::blog::insert_category))
    //             .service(web::scope("/user").service(crate::controllers::user::login)),
    //     )
    //     .await;

    //     let res = test::TestRequest::post()
    //         .uri("/user/login")
    //         .set_form(&serde_json::json!({
    //             "email": "hello@ludivinefarat.fr",
    //             "password": "root"
    //         }))
    //         .send_request(&mut app)
    //         .await;
    //     let cookie = res.headers().get(http::header::SET_COOKIE);

    //     assert!(cookie.is_some());
    //     assert!(res.status().is_success());

    //     let res = test::TestRequest::post()
    //         .uri("/api/blog/categories")
    //         .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
    //         .set_form(&serde_json::json!({
    //             "name": "Category 1",
    //             "is_visible": false,
    //             "is_seo": false
    //         }))
    //         .send_request(&mut app)
    //         .await;

    //     assert!(res.status().is_success());
    // }

    // #[actix_rt::test]
    // async fn test_insert_category_not_logged() {
    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();
    //     let mut app = test::init_service(
    //         App::new()
    //             .data(pool.clone())
    //             .service(web::scope("/api/blog").service(controllers::blog::insert_category)),
    //     )
    //     .await;

    //     let res = test::TestRequest::post()
    //         .uri("/api/blog/categories")
    //         .set_form(&serde_json::json!({
    //             "name": "Category 1",
    //             "is_visible": false,
    //             "is_seo": false
    //         }))
    //         .send_request(&mut app)
    //         .await;

    //     assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    // }

    // #[actix_rt::test]
    // async fn test_delete_category() {
    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();
    //     let mut app = test::init_service(
    //         App::new()
    //             .wrap(IdentityService::new(
    //                 CookieIdentityPolicy::new(&[0; 32])
    //                     .name("auth-cookie")
    //                     .secure(true),
    //             ))
    //             .data(pool.clone())
    //             .service(
    //                 web::scope("/api/blog")
    //                     .service(controllers::blog::insert_category)
    //                     .service(controllers::blog::delete_category),
    //             )
    //             .service(web::scope("/user").service(crate::controllers::user::login)),
    //     )
    //     .await;

    //     let res = test::TestRequest::post()
    //         .uri("/user/login")
    //         .set_form(&serde_json::json!({
    //             "email": "hello@ludivinefarat.fr",
    //             "password": "root"
    //         }))
    //         .send_request(&mut app)
    //         .await;
    //     let cookie = res.headers().get(http::header::SET_COOKIE);

    //     assert!(cookie.is_some());
    //     assert!(res.status().is_success());

    //     let res = test::TestRequest::post()
    //         .uri("/api/blog/categories")
    //         .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
    //         .set_form(&serde_json::json!({
    //             "name": "Category 1",
    //             "is_visible": false,
    //             "is_seo": false
    //         }))
    //         .send_request(&mut app)
    //         .await;
    //     let id: i16 = test::read_body_json(res).await;

    //     let res = test::TestRequest::delete()
    //         .uri(&format!("/api/blog/categories/{}", id))
    //         .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
    //         .send_request(&mut app)
    //         .await;

    //     assert!(res.status().is_success());
    // }

    // #[actix_rt::test]
    // async fn test_delete_category_not_logged() {
    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();
    //     let mut app = test::init_service(
    //         App::new()
    //             .wrap(IdentityService::new(
    //                 CookieIdentityPolicy::new(&[0; 32])
    //                     .name("auth-cookie")
    //                     .secure(true),
    //             ))
    //             .data(pool.clone())
    //             .service(
    //                 web::scope("/api/blog")
    //                     .service(controllers::blog::insert_category)
    //                     .service(controllers::blog::delete_category),
    //             )
    //             .service(web::scope("/user").service(crate::controllers::user::login)),
    //     )
    //     .await;

    //     let res = test::TestRequest::post()
    //         .uri("/user/login")
    //         .set_form(&serde_json::json!({
    //             "email": "hello@ludivinefarat.fr",
    //             "password": "root"
    //         }))
    //         .send_request(&mut app)
    //         .await;
    //     let cookie = res.headers().get(http::header::SET_COOKIE);

    //     assert!(cookie.is_some());
    //     assert!(res.status().is_success());

    //     let res = test::TestRequest::post()
    //         .uri("/api/blog/categories")
    //         .cookie(Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap())
    //         .set_form(&serde_json::json!({
    //             "name": "Category 1",
    //             "is_visible": false,
    //             "is_seo": false
    //         }))
    //         .send_request(&mut app)
    //         .await;
    //     let id: i16 = test::read_body_json(res).await;

    //     let res = test::TestRequest::delete()
    //         .uri(&format!("/api/blog/categories/{}", id))
    //         .send_request(&mut app)
    //         .await;

    //     assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    // }

    // #[actix_rt::test]
    // async fn test_insert_article() {
    //     use std::io::Read;
    //     use std::io::Write;

    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();

    //     let mut app = test::init_service(
    //         App::new()
    //             .wrap(IdentityService::new(
    //                 CookieIdentityPolicy::new(&[0; 32])
    //                     .name("auth-cookie")
    //                     .secure(true),
    //             ))
    //             .data(pool.clone())
    //             .service(web::scope("/api/blog").service(controllers::blog::insert_article))
    //             .service(web::scope("/user").service(crate::controllers::user::login)),
    //     )
    //     .await;

    //     let res = test::TestRequest::post()
    //         .uri("/user/login")
    //         .set_form(&serde_json::json!({
    //             "email": "hello@ludivinefarat.fr",
    //             "password": "root"
    //         }))
    //         .send_request(&mut app)
    //         .await;
    //     let cookie = res.headers().get(http::header::SET_COOKIE);

    //     assert!(cookie.is_some());
    //     assert!(res.status().is_success());

    //     let mut data: Vec<u8> = Vec::new();
    //     write!(data, "-----011000010111000001101001\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nLorem\r\n").unwrap();
    //     write!(data, "-----011000010111000001101001\r\nContent-Disposition: form-data; name=\"cover\"; filename=\"index.png\"\r\nContent-Type: image/png\r\n\r\n").unwrap();
    //     let mut f = std::fs::File::open("public/img/index.png").unwrap();
    //     f.read_to_end(&mut data).unwrap();
    //     write!(data, "\r\n").unwrap();
    //     write!(data, "-----011000010111000001101001\r\nContent-Disposition: form-data; name=\"blocks[]\"\r\n\r\n");
    //     write!(
    //         data,
    //         "{{\"title\":\"Lorem\",\"left_column\":true,\"order\":1}}\r\n"
    //     )
    //     .unwrap();
    //     write!(data, "-----011000010111000001101001--").unwrap();

    //     let cookie = Cookie::from_str(&cookie.unwrap().to_str().unwrap()).unwrap();
    //     let res = test::TestRequest::post()
    //         .uri("/api/blog/articles")
    //         .cookie(cookie)
    //         .set_payload(data)
    //         .header(
    //             header::CONTENT_TYPE,
    //             header::HeaderValue::from_static(
    //                 "multipart/form-data; boundary=---011000010111000001101001",
    //             ),
    //         )
    //         .send_request(&mut app)
    //         .await;

    //     assert!(res.status().is_success());
    // }

    // #[actix_rt::test]
    // async fn test_insert_article_not_logged() {
    //     use std::io::Read;
    //     use std::io::Write;

    //     dotenv().ok();

    //     let pool = create_pool().await.unwrap();

    //     let mut app = test::init_service(
    //         App::new()
    //             .wrap(IdentityService::new(
    //                 CookieIdentityPolicy::new(&[0; 32])
    //                     .name("auth-cookie")
    //                     .secure(true),
    //             ))
    //             .data(pool.clone())
    //             .service(web::scope("/api/blog").service(controllers::blog::insert_article)),
    //     )
    //     .await;

    //     let mut data: Vec<u8> = Vec::new();
    //     write!(data, "-----011000010111000001101001\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nLorem\r\n").unwrap();
    //     write!(data, "-----011000010111000001101001\r\nContent-Disposition: form-data; name=\"cover\"; filename=\"index.png\"\r\nContent-Type: image/png\r\n\r\n").unwrap();
    //     let mut f = std::fs::File::open("public/img/index.png").unwrap();
    //     f.read_to_end(&mut data).unwrap();
    //     write!(data, "\r\n").unwrap();
    //     write!(data, "-----011000010111000001101001\r\nContent-Disposition: form-data; name=\"blocks[]\"\r\n\r\n");
    //     write!(
    //         data,
    //         "{{\"title\":\"Lorem\",\"left_column\":true,\"order\":1}}\r\n"
    //     )
    //     .unwrap();
    //     write!(data, "-----011000010111000001101001--").unwrap();

    //     let res = test::TestRequest::post()
    //         .uri("/api/blog/articles")
    //         .set_payload(data)
    //         .header(
    //             header::CONTENT_TYPE,
    //             header::HeaderValue::from_static(
    //                 "multipart/form-data; boundary=---011000010111000001101001",
    //             ),
    //         )
    //         .send_request(&mut app)
    //         .await;

    //     assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);
    // }
}
