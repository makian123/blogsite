use actix_web::{get, post, delete, HttpRequest, web::Data, HttpResponse, cookie::{Cookie, Expiration, time::OffsetDateTime}};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use sha256::digest;

use crate::{app::{AppState, AppError}, database::models::{user::User}, auth::token::Token};

#[derive(Deserialize)]
struct DummyUser{
    pub username: String,
    pub password: String
}

/// Pipe for logging in as user
/// - url: `{domain}/user`
/// 
/// # HTTP request requirements
/// ## body
/// - json formatted string containing `username` and `password` keys
/// 
/// # Example
/// ```
/// let data = "{ username: \"Test username\", password: \"Test password\" }";
/// let request = actix_web::test::TestRequest::get()
///     .uri("localhost/user")
///     .set_payload(data)
///     .to_request();
/// ```
/// 
/// # Response
/// ## Ok
/// - set cookie header containing login token
/// ## Error
/// - Bad request
/// - Unauthorized
/// - Internal server error
#[get("/user")]
pub async fn login(_req: HttpRequest, req_body: String, app_state: Data<AppState>) -> Result<HttpResponse, AppError>{
    let credentials: Value = serde_json::from_str(req_body.trim()).unwrap();
    if credentials.get("username").is_none() || credentials.get("password").is_none(){
        return Err(AppError::BadRequest); 
    }
    
    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let username = credentials.get("username").unwrap().as_str().unwrap().to_string();
    let pw = digest(credentials.get("password").unwrap().as_str().unwrap().to_string());

    let user = User::find_user_by_username(&psql_conn, &username).ok_or(AppError::UnauthorizedError)?;

    if user.pass != pw {
        return Err(AppError::UnauthorizedError); 
    }

    let token = Token::new(&mut redis_conn, &user.id);
    let cookie = Cookie::build("token", token)
        .path("/")
        .expires(Expiration::DateTime(OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + 180).unwrap()))
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).finish())
}

/// Pipe for creating an user
/// - url: `{domain}/user`
/// 
/// # HTTP request requirements
/// ## body
/// - json formatted string containing `username` and `password` keys
/// - `password` must be at least 10 characters long
/// 
/// # Example
/// ```
/// let data = "{ username: \"Test username\", password: \"Test password\" }";
/// let request = actix_web::test::TestRequest::post()
///     .uri("localhost/user")
///     .set_payload(data)
///     .to_request();
/// ```
/// 
/// # Response
/// ## Ok
/// ## Error
/// - Bad request
#[post("/user")]
pub async fn create_new_user(req_body: String, app_state: Data<AppState>) -> Result<HttpResponse, AppError>{
    let mut user = serde_json::from_str::<DummyUser>(&req_body).map_err(|_| AppError::BadRequest)?;
    let conn = app_state.psql_pool.clone().get().unwrap();

    user.password = user.password.trim().to_string();

    if user.password.len() < 10 { 
        return Err(AppError::BadRequest); 
    }
    if User::find_user_by_username(&conn, &user.username).is_some() {
        return Err(AppError::BadRequest); 
    }

    let _final_user = User::new(&conn, &user.username, &digest(user.password), true);

    Ok(HttpResponse::Ok().finish())
}

/// Pipe for deleting an user
/// - url: `{domain}/user/{username}`
/// 
/// # HTTP request requirements
/// - `{username}` value as parameter
/// ## header
/// - cookie named `token` containing login token
/// 
/// # Example
/// ```
/// let cookie = CookieBuilder::new("token", "test_token").finish();
/// let request = actix_web::test::TestRequest::delete()
///     .uri("localhost/user/test_user")
///     .cookie(cookie)
///     .to_request();
/// ```
/// 
/// # Response
/// ## Ok
/// ## Error
/// - Bad request
/// - Unauthorized
#[delete("/user/{username}")]
pub async fn delete_an_user(req: HttpRequest, app_state: Data<AppState>) -> Result<HttpResponse, AppError> {
    let username = req.match_info().query("username").to_string();
    let token = req.cookie("token").ok_or(AppError::BadRequest)?.value().to_string();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let user_id = Token::find(&mut redis_conn, &token)?;

    let conn = app_state.psql_pool.clone().get().unwrap();
    let user = User::find_by_id(&conn, &user_id)?;
    if user.username != username { 
        return Err(AppError::Forbidden);
    }

    Token::delete(&mut redis_conn, &token);
    user.delete(&conn);

    Ok(HttpResponse::Ok().finish())
}
