use actix_web::{get, post, delete, HttpRequest, web::Data, Responder, HttpResponse, cookie::{Cookie, Expiration, time::OffsetDateTime}};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use sha256::digest;

use crate::{app::AppState, database::models::{User, Blog}, auth::token::Token};

#[derive(Deserialize)]
struct DummyUser{
    pub username: String,
    pub password: String
}

//User routes
#[get("/login")]
pub async fn login(_req: HttpRequest, req_body: String, app_state: Data<AppState>) -> impl Responder{
    let credentials: Value = serde_json::from_str(&req_body).unwrap();
    if credentials.get("username").is_none() || credentials.get("password").is_none(){ return HttpResponse::BadRequest().body("Bad request"); }
    
    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let username = credentials.get("username").unwrap().as_str().unwrap().to_string();
    let pw = digest(credentials.get("password").unwrap().as_str().unwrap().to_string());

    let user = User::find_user_by_username(&psql_conn, &username);
    if user.is_none() { return HttpResponse::Unauthorized().body("Username not found"); }
    let user = user.unwrap();

    if user.pass != pw { return HttpResponse::Unauthorized().body("Password not good"); }

    let token = Token::new(&mut redis_conn, &user.id);
    let cookie = Cookie::build("token", token)
        .path("/")
        .expires(Expiration::DateTime(OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + 180).unwrap()))
        .finish();

    HttpResponse::Ok().cookie(cookie).finish()
}
#[post("/user")]
pub async fn create_new_user(req_body: String, app_state: Data<AppState>) -> impl Responder{
    let user = serde_json::from_str::<DummyUser>(&req_body);
    if user.is_err() { return HttpResponse::BadRequest(); }
    let user = user.unwrap();
    let conn = app_state.psql_pool.clone().get().unwrap();

    if User::find_user_by_username(&conn, &user.username).is_some() { return HttpResponse::BadRequest(); }
    if user.password.len() == 0 || user.password.len() < 6 { return HttpResponse::BadRequest(); }

    let _final_user = User::new(&conn, &user.username, &digest(user.password), true);

    HttpResponse::Ok()
}
#[delete("/user")]
pub async fn delete_an_user(req: HttpRequest, app_state: Data<AppState>) -> impl Responder {
    let username = req.match_info().query("username").to_string();
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest(); }
    let token = token.unwrap().value().to_string();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let user_id = Token::find(&mut redis_conn, &token);
    if user_id.is_err() { return HttpResponse::BadRequest(); }
    let user_id = user_id.unwrap();

    let conn = app_state.psql_pool.clone().get().unwrap();
    let user = User::find_by_id(&conn, &user_id);
    if user.is_none() { return HttpResponse::BadRequest(); }
    let user = user.unwrap();
    if user.username != username { return HttpResponse::BadRequest(); }
    Blog::delete_by_user_id(&conn, &user.id);
    Token::delete(&mut redis_conn, &token);
    user.delete(&conn);

    HttpResponse::Ok()
}
