use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration},
    delete, get, post,
    web::Data,
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use sha256::digest;

use crate::{
    app::{AppError, AppState},
    auth::token::Token,
    database::models::user::*,
};

#[derive(Deserialize)]
struct DummyUser {
    pub username: String,
    pub password: String,
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
pub async fn login(
    _req: HttpRequest,
    req_body: String,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let credentials: Value = serde_json::from_str(req_body.trim()).unwrap();
    if credentials.get("username").is_none() || credentials.get("password").is_none() {
        return Err(AppError::BadRequest);
    }

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let username = credentials
        .get("username")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let pw = digest(
        credentials
            .get("password")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    );

    let user =
        User::find_by_username(Some(&psql_conn), &username).ok_or(AppError::UnauthorizedError)?;

    if user.pass != pw {
        return Err(AppError::UnauthorizedError);
    }

    let token = Token::new(&mut redis_conn, &user.id);
    let cookie = Cookie::build("token", token)
        .path("/")
        .expires(Expiration::DateTime(
            OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + 180).unwrap(),
        ))
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
pub async fn create_new_user(
    req_body: String,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let mut user =
        serde_json::from_str::<DummyUser>(&req_body).map_err(|_| AppError::BadRequest)?;
    let conn = app_state.psql_pool.clone().get().unwrap();

    user.password = user.password.trim().to_string();

    if user.password.len() < 10 {
        return Err(AppError::BadRequest);
    }
    if User::find_by_username(Some(&conn), &user.username).is_some() {
        return Err(AppError::BadRequest);
    }

    let _final_user = User::new(Some(&conn), &user.username, &digest(user.password), true);

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
pub async fn delete_an_user(
    req: HttpRequest,
    app_state: Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let username = req.match_info().query("username").to_string();
    let token = req
        .cookie("token")
        .ok_or(AppError::BadRequest)?
        .value()
        .to_string();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let user_id = Token::find(&mut redis_conn, &token)?;

    let conn = app_state.psql_pool.clone().get().unwrap();
    let user = User::find_by_id(Some(&conn), &user_id)?;
    if user.username != username {
        return Err(AppError::Forbidden);
    }

    Token::delete(&mut redis_conn, &token);
    user.delete(Some(&conn));

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, test::call_service, App};

    #[actix_rt::test]
    async fn test_user_login() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::login),
        )
        .await;

        let payload = "{ \"username\": \"marko13\", \"password\": \"gaser_marko\"}";
        let req = test::TestRequest::get()
            .uri("/user")
            .insert_header(actix_web::http::header::ContentType::json())
            .app_data(actix_web::web::Data::new(appstate.clone()))
            .set_payload(payload)
            .to_request();

        let resp = call_service(&app, req).await;
        debug_assert!(resp.status().is_success());

        let cookie = resp.headers().get("set-cookie");
        debug_assert!(cookie != None);

        let token = std::str::from_utf8(cookie.unwrap().as_bytes()).unwrap();
        Token::delete(
            &mut appstate.redis_pool.get().unwrap(),
            &String::from(Cookie::parse(token).unwrap().value()),
        );
    }

    //#[actix_rt::test]
    async fn test_user_create() {
        let appstate = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(appstate.clone()))
                .service(super::create_new_user)
                .service(super::login),
        )
        .await;

        let payload = "{ \"username\": \"Test_user123\", \"password\": \"test_user123\"}";
        let req = test::TestRequest::post()
            .uri("/user")
            .insert_header(actix_web::http::header::ContentType::json())
            .app_data(Data::new(appstate.clone()))
            .set_payload(payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        debug_assert!(resp.status().is_success());

        let req = test::TestRequest::get()
            .uri("/user")
            .insert_header(actix_web::http::header::ContentType::json())
            .app_data(Data::new(appstate.clone()))
            .set_payload(payload)
            .to_request();

        let resp = call_service(&app, req).await;
        debug_assert!(resp.status().is_success());

        let user = User::find_by_username(
            Some(&appstate.psql_pool.get().unwrap()),
            &"Test_user123".to_string(),
        );
        debug_assert!(user.is_some());

        User::delete(&user.unwrap(), Some(&appstate.psql_pool.get().unwrap()));
    }
}
