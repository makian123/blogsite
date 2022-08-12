use crate::{
    app::{AppError, AppState},
    auth::token::Token,
    database::models::{blog::*, comment::*, user::*},
};
use actix_web::{delete, get, post, web::Data, HttpRequest, HttpResponse};

/// Pipe for creating a comment
/// - url: `{domain}/blogs/{blog_id}/comment`
///
/// # HTTP request requires
/// - `{blog_id}` as a parameter
///
/// ## body
/// - a string of the comment text
///
/// # Example
/// ```
/// let comment = "comment text";
/// let cookie = CookieBuilder::new("token", "test_token").finish();
/// let request = actix_web::test::TestRequest::post()
///     .uri("localhost/blogs/blog_id/comment")
///     .set_payload(comment)
///     .cookie(cookie)
///     .to_request();
/// ```
///
/// # Response
/// ## Ok
/// ## Error
/// - Unauthorized
/// - Bad request
/// - Internal server errror
#[post("/blogs/{blog_id}/comment")]
pub async fn create_comment(
    req: HttpRequest,
    req_body: String,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let token = req
        .cookie("token")
        .ok_or(AppError::UnauthorizedError)?
        .value()
        .to_string();
    let blog_id = req.match_info().query("blog_id").parse::<i32>().unwrap();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    //Checks if blog exists
    Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;

    let user_id = Token::find(&mut redis_conn, &token)?;
    let comment = Comment::new(&psql_conn, blog_id, &user_id, &req_body)
        .ok_or(AppError::InternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

/// Pipe for getting comments from blog
/// - url: `{domain}/blogs/{blog_id}/comments`
///
/// # HTTP request requirements
/// - `{blog_id}` as url parameter
///
/// # Example
/// ```
/// let request = actix_web::test::TestRequest::get()
///     .uri("localhost/blogs/blog_id/comments")
///     .to_request();
/// ```
///
/// # Response
/// ## Ok
/// - json formatted string of the blog [comments](Comment) in the string
/// ```
/// [
///     {
///         "id":"ef7a71b8-53bf-4c01-a3ad-39c332adbb39",
///         "user_id":"e60a0f7b-381c-46b7-8736-1f204b329727",
///         "blog_id":73,"created_at":"2022-08-12T06:05:31.097180",
///         "body":"Comment body 1"
///     },
///     {
///         "id":"30d177a9-b678-4b2f-85fd-33ad960fadd10ace0",
///         "user_id":"e60a0f7b-381c-46b7-8736-1fa21ea5",
///         "blog_id":73,
///         "created_at":"2022-08-12T06:04:59.580527",
///         "body":"Comment body 2"
///     }
/// ]
/// ```
/// ## Error
/// - Bad request
/// - Internal server error
#[get("/blogs/{blog_id}/comments")]
pub async fn get_comments(
    req: HttpRequest,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let blog_id = req.match_info().query("blog_id").parse::<i32>()?;

    let psql_conn = app_state.psql_pool.clone().get().unwrap();

    let comments = Comment::find_by_blog(&psql_conn, blog_id).unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string(&comments).unwrap()))
}

/// Pipe for deleting a comment from a certain post
/// - url: `{domain}/blogs/{blog_id}/comments/{comment_id}`
///
/// # HTTP request requires
/// - `{blog_id}` and `{comment_id}` as url parameters
///
/// ## header
/// - cookie named `token` containing login token
///
/// # Example
/// ```
/// let request = actix_web::test::TestRequest::delete()
///     .uri("localhost/blogs/blog_id/comments/comment_id")
///     .set_payload(comment)
///     .cookie(cookie)
///     .to_request();
/// ```
///
/// # Response
/// ## Ok
/// ## Error
/// - Unauthorized
/// - Bad request
/// - Forbidden
/// - Internal server error
#[delete("/blogs/{blog_id}/comments/{comment_id}")]
pub async fn delete_comment(
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

    let comment_id = req.match_info().query("comment_id").to_string();
    let comment = Comment::find_by_id(&psql_conn, &comment_id).ok_or(AppError::BadRequest)?;

    if comment.user_id != user.id && !user.is_admin {
        return Err(AppError::Forbidden);
    }

    Comment::delete(&psql_conn, &comment_id);

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use actix_web::{body, cookie::CookieBuilder, test, App};
    use sha256::digest;

    use super::*;

    #[actix_rt::test]
    async fn test_new_comment() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::create_comment),
        )
        .await;

        let usr = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test user123"),
            &digest("asd123"),
            false,
        )
        .unwrap();
        let token = Token::new(&mut appstate.redis_pool.get().unwrap(), &usr.id);
        let cookie = CookieBuilder::new("token", &token).path("/").finish();
        let blog = Blog::new(
            &appstate.psql_pool.get().unwrap(),
            &usr,
            &String::from("Test title"),
            &String::from("Test body"),
            None,
        )
        .unwrap();

        let req = test::TestRequest::post()
            .uri(format!("/blogs/{}/comment", blog.id).as_str())
            .app_data(appstate.clone())
            .cookie(cookie)
            .set_payload(String::from("test_comment"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        debug_assert!(resp.status().is_success());
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let data = std::str::from_utf8(&body).unwrap();

        debug_assert!(
            Comment::find_by_id(&appstate.psql_pool.get().unwrap(), &String::from(data)).is_some()
        );

        usr.delete(Some(&appstate.psql_pool.get().unwrap()));
    }

    #[actix_rt::test]
    async fn test_get_comments() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::get_comments),
        )
        .await;

        let usr = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test user123"),
            &digest("asd123"),
            false,
        )
        .unwrap();
        Token::new(&mut appstate.redis_pool.get().unwrap(), &usr.id);
        let blog = Blog::new(
            &appstate.psql_pool.get().unwrap(),
            &usr,
            &String::from("Test title"),
            &String::from("Test body"),
            None,
        )
        .unwrap();
        Comment::new(
            &appstate.psql_pool.get().unwrap(),
            blog.id,
            &usr.id,
            &String::from("Test comment"),
        );

        let req = test::TestRequest::get()
            .uri(format!("/blogs/{}/comments", blog.id).as_str())
            .to_request();
        let resp = test::call_service(&app, req).await;
        debug_assert!(resp.status().is_success());
        debug_assert!(body::to_bytes(resp.into_body()).await.unwrap().len() > 0);

        usr.delete(Some(&appstate.psql_pool.get().unwrap()));
    }

    #[actix_rt::test]
    async fn test_delete_comment() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::delete_comment),
        )
        .await;

        let usr = User::new(
            Some(&appstate.psql_pool.get().unwrap()),
            &String::from("Test user123"),
            &digest("asd123"),
            false,
        )
        .unwrap();
        let token = Token::new(&mut appstate.redis_pool.get().unwrap(), &usr.id);
        let cookie = CookieBuilder::new("token", &token).path("/").finish();
        let blog = Blog::new(
            &appstate.psql_pool.get().unwrap(),
            &usr,
            &String::from("Test title"),
            &String::from("Test body"),
            None,
        )
        .unwrap();
        let comment = Comment::new(
            &appstate.psql_pool.get().unwrap(),
            blog.id,
            &usr.id,
            &String::from("Test comment"),
        )
        .unwrap();

        let req = test::TestRequest::delete()
            .uri(format!("/blogs/{}/comments/{}", blog.id, comment.id).as_str())
            .app_data(appstate.clone())
            .cookie(cookie)
            .to_request();
        let resp = test::call_service(&app, req).await;
        debug_assert!(resp.status().is_success());
        debug_assert!(
            Comment::find_by_id(&appstate.psql_pool.get().unwrap(), &comment.id).is_none()
        );

        usr.delete(Some(&appstate.psql_pool.get().unwrap()));
    }
}
