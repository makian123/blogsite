use std::{fs, io::Write, path::PathBuf};

use crate::{
    app::{AppError, AppState},
    auth::token::Token,
    database::models::{blog::*, like::*, user::*},
};
use actix_multipart::Multipart;
use actix_web::{
    delete, get, post, put,
    web::{self, Data},
    HttpRequest, HttpResponse,
};
use futures::{stream::StreamExt as _, TryStreamExt};
use serde_json::Value;
use uuid::Uuid;

fn extract_extension(buffer: &[u8]) -> String {
    let mut ret = String::new();
    let mut started = false;

    for i in 0..buffer.len() {
        if match buffer[i] {
            65..=90 => {
                if !started {
                    started = true;
                }
                ret.push(buffer[i] as char);
                true
            }
            _ => {
                if started {
                    false
                } else {
                    true
                }
            }
        } == false
        {
            break;
        }
    }

    ret.to_lowercase()
}

async fn parse_multipart(payload: &mut Multipart) -> Result<(String, String, String), AppError> {
    let (mut title, mut body, mut filename) = (String::new(), String::new(), String::new());

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition();

        if content_type.get_name().is_some() {
            match content_type.get_name().unwrap() {
                "file" => {
                    let file_name = Uuid::new_v4();
                    let mut p = PathBuf::new();
                    p.push(format!("images/{}", file_name.clone().to_string()));
                    let mut cloned_path = p.clone();
                    let real_path = p.clone();
                    filename.clear();
                    let mut file = web::block(|| std::fs::File::create(p))
                        .await
                        .unwrap()
                        .unwrap();

                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();

                        file = web::block(move || file.write_all(&data).map(|_| file))
                            .await
                            .unwrap()
                            .unwrap();
                    }

                    let the_file = fs::read(real_path.clone()).unwrap();
                    let ret = extract_extension(&the_file[..]);

                    if ret.is_empty() || the_file.len() == 0 {
                        fs::remove_file(real_path.clone())?;
                        continue;
                    }

                    cloned_path.set_extension(ret.clone());
                    let _return = fs::rename(real_path, cloned_path.clone());

                    filename.push_str(format!("{}.{}", Uuid::to_string(&file_name), &ret).as_str());
                }
                "title" => {
                    title.clear();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        title.push_str(std::str::from_utf8(&data).unwrap());
                    }
                }
                "body" => {
                    body.clear();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        body.push_str(std::str::from_utf8(&data).unwrap());
                    }
                }
                _ => {}
            };
        }
    }

    if title.len() == 0 && body.len() == 0 {
        return Err(AppError::BadRequest);
    }

    Ok((title, body, filename))
}

/// Pipe for creating a new blog, it is of type multipart
/// - url: `{domain}/blog`
///
/// # HTTP request requirements
/// ## header
/// - cookie with name `token`, containing the login token
/// ## body
/// - file: [fs::File] (optional) - image we are uploading
/// - title: [String] - title we wish to name our blog
/// - body: [String] - body of the blog
///
/// # Response
/// ## Ok
/// - filename of the file uploaded (if there was one)
/// ```
/// "asfs12iudk2kdssab.jpg"
/// ```
/// ## Error
/// - Bad request
/// - Unauthorized
/// - Internal server error
#[post("/blog")]
pub async fn create_new_blog(
    req: HttpRequest,
    app_state: Data<AppState>,
    mut mp: Multipart,
) -> Result<HttpResponse, AppError> {
    let token = req
        .cookie("token")
        .ok_or(AppError::UnauthorizedError)?
        .value()
        .to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    let user_id = Token::find(&mut redis_conn, &token)?;
    let user = User::find_by_id(Some(&psql_conn), &user_id)?;
    let (title, body, filename) = parse_multipart(&mut mp).await?;

    Blog::new(
        &psql_conn,
        &user,
        &title,
        &body,
        if filename.len() == 0 {
            None
        } else {
            Some(&filename)
        },
    )?;

    Ok(HttpResponse::Ok().body(filename))
}

/// Pipe for getting blogs with the specified username
/// - url: `{domain}/blogs/{username}`
///
/// # HTTP request requires
/// - `{username}` as a parameter
///
/// # Example
/// ```
/// let request = actix_web::test::TestRequest::get()
///     .uri("localhost/blogs/test_username")
///     .to_request();
/// ```
///
/// # Response
/// ## Ok
/// - json formatted string of all [blogs](Blog) created by user
/// ```
/// [
///     {
///         "id":73,
///         "title":"Blog title",
///         "body":"Blog body",
///         "image_id":"b600b24f-9414-4009-b538-8b9ac77292be.png",
///         "created_by":"e60a0f7b-381c-46b7-8736-1f204b329727",
///         "created_at":"2022-08-04T09:22:30.664361",
///         "updated_at":"2022-08-04T09:22:30.664361",
///         "likes":0
///     }
/// ]
/// ```
/// ## Error
/// - Bad request
#[get("/blogs/{username}")]
pub async fn get_blogs_by_user(
    req: HttpRequest,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let username = req.match_info().query("username").to_string();

    let conn = app_state.psql_pool.clone().get().unwrap();
    let user = User::find_by_username(Some(&conn), &username).ok_or(AppError::BadRequest)?;

    let posts = Blog::get_by_creator_id(&conn, &user.id);
    Ok(HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap()))
}

/// Pipe for editing a certain blog parameter
/// - url: `{domain}/blogs/{blog_id}`
///
/// # HTTP request requirements
/// - `{blog_id}` as a paremeter
///
/// ## header
/// - cookie with name `token`, containing the login token
///
/// ## body
/// - json with the specified fields we are changing: 'title' and/or 'body'
///
/// # Example
/// ```
/// let edit_title = "{ title: \"Test title\" }";
/// let edit_body = "{ body: \"Test body\" }";
/// let edit_all = "{ title: \"Test title\", body: \"Test body\" }";
///
/// let cookie = CookieBuilder::new("token", "test_token").finish();
/// let edit_title_request = actix_web::test::TestRequest::put()
///     .uri("localhost/blogs/test_blog_id")
///     .cookie(cookie)
///     .set_payload(edit_title)
///     .to_request();
///
/// let edit_body_request = actix_web::test::TestRequest::put()
///     .uri("localhost/blogs/test_blog_id")
///     .cookie(cookie)
///     .set_payload(edit_body)
///     .to_request();
///
/// let edit_all_request = actix_web::test::TestRequest::put()
///     .uri("localhost/blogs/test_blog_id")
///     .cookie(cookie)
///     .set_payload(edit_all)
///     .to_request();
/// ```
///
/// # Response
/// ## OK
/// ## Error
/// - Unauthorized
/// - Bad request
/// - Internal server error
#[put("/blogs/{blog_id}")]
pub async fn edit_blogs(
    req: HttpRequest,
    req_body: String,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let token = req
        .cookie("token")
        .ok_or(AppError::UnauthorizedError)?
        .value()
        .to_string();

    //Checks for request body, if there's none, throw bad request
    let updated_blog =
        serde_json::from_str::<Value>(&req_body).map_err(|_| AppError::BadRequest)?;

    //Starts a db and tries to find user from supplied id
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let psql_conn = app_state.psql_pool.clone().get().unwrap();

    let _user_id = Token::find(&mut redis_conn, &token)?;
    let blog_id = req.match_info().query("blog_id").parse::<i32>()?;

    //Tries to find a blog posted by that user with the id
    //if no blog found throw bad request
    let mut blog = Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;
    //Tries to parse the json values into normal values if they exist
    let title = updated_blog.get("title");
    let mut title_optional = String::new();
    let body = updated_blog.get("body");
    let mut body_optional = String::new();

    if title.is_some() {
        title_optional = title.unwrap().as_str().unwrap().to_string();
    }
    if body.is_some() {
        body_optional = body.unwrap().as_str().unwrap().to_string();
    }

    blog.edit(
        &psql_conn,
        match title {
            Some(_x) => Some(&title_optional),
            None => None,
        },
        match body {
            Some(_x) => Some(&body_optional),
            None => None,
        },
        None,
    );

    Ok(HttpResponse::Ok().finish())
}

/// Pipe for liking or disliking an post, if the post isn't liked by the user, it will become liked.
/// However, if the post is already liked, the like is removed
/// - url: `{domain}/blogs/{blog_id}/like`
///
/// # HTTP request requirements
/// - `{blog_id}` as a paremeter
///
/// ## header
/// - cookie with name `token`, containing the login token
///
/// # Example
/// ```
/// let cookie = CookieBuilder::new("token", "test_token").finish();
/// let like_request = actix_web::test::TestRequest::put()
///     .uri("localhost/blogs/blog_id/like")
///     .cookie(cookie)
///     .to_request();
/// ```
///
/// # Response
/// ## Ok
/// ## Error
/// - Unauthorized
/// - Internal server error
/// - Bad request
#[put("/blogs/{blog_id}/like")]
pub async fn like_a_blog(
    req: HttpRequest,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let token = req
        .cookie("token")
        .ok_or(AppError::UnauthorizedError)?
        .value()
        .to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let blog_id = req.match_info().query("blog_id").parse().unwrap();

    //Confirms user is logged in
    Token::find(&mut redis_conn, &token)?;
    let user_id = Token::find(&mut redis_conn, &token).unwrap();

    let mut blog = Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;

    let like = Like::new(&psql_conn, &user_id, blog_id);
    if like.is_none() {
        blog.edit(&psql_conn, None, None, Some(blog.likes - 1));
        Like::delete(&psql_conn, &user_id, blog_id);
        return Ok(HttpResponse::Ok().finish());
    }
    blog.edit(&psql_conn, None, None, Some(blog.likes + 1));

    Ok(HttpResponse::Ok().finish())
}

/// Pipe for deleting an blog, this also deletes the image specified with the blog if it is present
/// - url: `{domain}/blogs{blog_id}`
///
/// # HTTP request requirements
/// - `{blog_id}` as parameter
///
/// ## header
/// - token named `token` containing login token
///
/// # Example
/// ```
/// let cookie = CookieBuilder::new("token", "test_token").finish();
/// let like_request = actix_web::test::TestRequest::delete()
///     .uri("localhost/blogs/blog_id")
///     .cookie(cookie)
///     .to_request();
/// ```
///
/// # Response
/// ## Ok
/// ## Error
/// - Unauthorized
/// - Bad request
/// - Internal server error
/// - Forbidden
#[delete("/blogs/{blog_id}")]
pub async fn delete_blog(
    req: HttpRequest,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let token = req
        .cookie("token")
        .ok_or(AppError::UnauthorizedError)?
        .value()
        .to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    let user_id = Token::find(&mut redis_conn, &token)?;
    let user = User::find_by_id(Some(&psql_conn), &user_id)?;

    let blog_id = req.match_info().query("blog_id").parse::<i32>()?;
    let blog = Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;

    if user.id != blog.created_by && !user.is_admin {
        return Err(AppError::Forbidden);
    }
    Blog::delete_by_id(&psql_conn, blog.id);

    Ok(HttpResponse::Ok().finish())
}

/// Pipe for getting an image
/// - url: `{domain}/images/{image_name}`
///
/// # HTTP request requirements
/// - `{image_name}` as parameter
///
/// # Example
/// ```
/// let like_request = actix_web::test::TestRequest::get()
///     .uri("localhost/images/image_name")
///     .to_request();
/// ```
///
/// # Response
/// ## Ok
/// - image file in the body
/// ## Error
/// - Bad request
/// - Internal server error
#[get("/images/{image_name}")]
pub async fn get_image(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let img_name = req.match_info().query("image_name");

    let mut file_path = PathBuf::new();
    file_path.push("images");
    file_path.push(img_name);

    let file = fs::read(file_path)?;

    Ok(HttpResponse::Ok().body(file))
}

#[cfg(test)]
mod tests {
    use actix_web::cookie::CookieBuilder;
    use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
    use actix_web::test::{self, call_service};
    use actix_web::App;
    use sha256::digest;
    use tokio_stream;

    use super::*;

    //#[actix_rt::test]
    async fn test_blog_create() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::create_new_blog),
        )
        .await;

        let usr = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test_user123"),
            &digest("test_password123"),
            false,
        )
        .unwrap();
        let token = Token::new(&mut appstate.redis_pool.get().unwrap(), &usr.id);
        let cookie = CookieBuilder::new("token", token).finish();

        let mut header_map = HeaderMap::new();
        header_map.insert(
            HeaderName::from_static("title"),
            HeaderValue::from_static("Test title"),
        );
        header_map.insert(
            HeaderName::from_static("body"),
            HeaderValue::from_static("Test body"),
        );

        let newval = Box::new("asd123");
        let strm = tokio_stream::empty();

        let mp = Multipart::new(&header_map, strm);

        let req = test::TestRequest::post()
            .uri("/blog")
            .cookie(cookie)
            .app_data(appstate.clone())
            .insert_header((actix_web::http::header::CONTENT_TYPE, "multipart/form-data"))
            .to_request();
        let resp = call_service(&app, req).await;
    }

    #[actix_rt::test]
    async fn test_get_blogs_by_user() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::get_blogs_by_user),
        )
        .await;

        let usr = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test_user123"),
            &digest("test_password123"),
            false,
        )
        .unwrap();
        let token = Token::new(&mut appstate.redis_pool.get().unwrap(), &usr.id);
        let _blog = Blog::new(
            &appstate.psql_pool.get().unwrap(),
            &usr,
            &String::from("Test title"),
            &String::from("Test body"),
            None,
        )
        .unwrap();
        let cookie = CookieBuilder::new("token", token).finish();

        let req = test::TestRequest::get()
            .uri("/blogs/Test_user123")
            .insert_header(actix_web::http::header::ContentType::form_url_encoded())
            .cookie(cookie.clone())
            .to_request();

        let resp = call_service(&app, req).await;
        if !resp.status().is_success() {
            usr.delete(Some(&appstate.psql_pool.get().unwrap()));
            panic!();
        }

        usr.delete(Some(&appstate.psql_pool.get().unwrap()));

        let req = test::TestRequest::get()
            .uri("/blogs/Test_user123")
            .insert_header(actix_web::http::header::ContentType::form_url_encoded())
            .cookie(cookie)
            .to_request();

        let resp = call_service(&app, req).await;
        pretty_assertions::assert_ne!(resp.status().as_u16(), 200);

        let mut ctr = 0;
        let mut user = User::find_by_id(Some(&appstate.psql_pool.get().unwrap()), &usr.id);
        while ctr < 10 && user.is_ok() {
            user.unwrap()
                .delete(Some(&appstate.psql_pool.get().unwrap()));
            user = User::find_by_id(Some(&appstate.psql_pool.get().unwrap()), &usr.id);
            actix::clock::sleep(std::time::Duration::from_millis(500)).await;
            ctr += 1;
        }
        assert!(user.is_err());
    }

    #[actix_rt::test]
    async fn test_blog_edit() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::edit_blogs),
        )
        .await;

        let user = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test_user123"),
            &digest("test_password123"),
            false,
        )
        .unwrap();
        let token = Token::new(&mut appstate.redis_pool.get().unwrap(), &user.id);
        let cookie = CookieBuilder::new("token", token).finish();
        let blog = Blog::new(
            &appstate.psql_pool.get().unwrap(),
            &user,
            &String::from("Test title"),
            &String::from("Test body"),
            None,
        )
        .unwrap();

        let payload = "{ \"title\": \"edited test title\", \"body\": \"edited test body\" }";

        let req = test::TestRequest::put()
            .uri(format!("/blogs/{}", blog.id).as_str())
            .insert_header(actix_web::http::header::ContentType::json())
            .set_payload(payload)
            .cookie(cookie)
            .to_request();

        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());
        let blog = Blog::get_by_id(&appstate.psql_pool.get().unwrap(), blog.id).unwrap();

        pretty_assertions::assert_eq!(blog.title, String::from("edited test title"));
        pretty_assertions::assert_eq!(blog.body, String::from("edited test body"));

        user.delete(Some(&appstate.psql_pool.get().unwrap()));
    }

    #[actix_rt::test]
    async fn test_blog_like() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::like_a_blog),
        )
        .await;

        let user = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test_user123"),
            &digest("test_password123"),
            false,
        )
        .unwrap();
        let token = Token::new(&mut appstate.redis_pool.get().unwrap(), &user.id);
        let cookie = CookieBuilder::new("token", token).finish();
        let blog = Blog::new(
            &appstate.psql_pool.get().unwrap(),
            &user,
            &String::from("Test title"),
            &String::from("Test body"),
            None,
        )
        .unwrap();

        let req = test::TestRequest::put()
            .uri(format!("/blogs/{}/like", blog.id).as_str())
            .app_data(actix_web::web::Data::new(appstate.clone()))
            .cookie(cookie.clone())
            .to_request();

        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());

        if let Some(found_blog) = Blog::get_by_id(&appstate.psql_pool.get().unwrap(), blog.id) {
            assert!(found_blog.likes == 1);
        } else {
            panic!();
        }

        let req = test::TestRequest::put()
            .uri(format!("/blogs/{}/like", blog.id).as_str())
            .app_data(appstate.clone())
            .cookie(cookie.clone())
            .to_request();

        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());

        if let Some(found_blog) = Blog::get_by_id(&appstate.psql_pool.get().unwrap(), blog.id) {
            assert!(found_blog.likes == 0);
        } else {
            panic!();
        }
        user.delete(Some(&appstate.psql_pool.get().unwrap()));
    }

    #[actix_rt::test]
    async fn test_blog_delete() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::delete_blog),
        )
        .await;

        let user = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test_user123"),
            &digest("test_password123"),
            false,
        )
        .unwrap();
        let token = Token::new(&mut appstate.redis_pool.get().unwrap(), &user.id);
        let cookie = CookieBuilder::new("token", token).finish();
        let blog = Blog::new(
            &appstate.psql_pool.get().unwrap(),
            &user,
            &String::from("Test title"),
            &String::from("Test body"),
            None,
        )
        .unwrap();

        let req = test::TestRequest::delete()
            .uri(format!("/blogs/{}", blog.id).as_str())
            .app_data(appstate.clone())
            .cookie(cookie)
            .to_request();
        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());

        assert!(Blog::get_by_id(&appstate.psql_pool.get().unwrap(), blog.id).is_none());
        user.delete(Some(&appstate.psql_pool.get().unwrap()));
    }

    #[actix_rt::test]
    async fn test_get_image() {
        let app = test::init_service(App::new().service(super::get_image)).await;

        let req = test::TestRequest::get()
            .uri("/images/asd213.asd")
            .to_request();

        pretty_assertions::assert_ne!(call_service(&app, req).await.status().is_success(), true);
        let mut path = PathBuf::new();
        path.push("images/asd213.asd");

        std::fs::write(path.clone(), &"asd123").unwrap();

        let req = test::TestRequest::get()
            .uri("/images/asd213.asd")
            .to_request();
        if call_service(&app, req).await.status().is_success() != true {
            std::fs::remove_file(path).unwrap();
            panic!();
        }

        std::fs::remove_file(path).unwrap();
    }
}
