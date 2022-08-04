use actix_web::{delete, put, HttpRequest, web::Data, Responder, HttpResponse, cookie::{Cookie, Expiration, time::OffsetDateTime}};
use chrono::Utc;

use crate::{app::AppState, auth::token::Token};

/// Pipe for deauthorizing a token and removing it from the database
/// - url: `{domain}/api/deauth`
/// 
/// # HTTP request requirements
/// ## header
/// - cookie named `token` containing login token
/// 
/// # Example
/// ```
/// let cookie = CookieBuilder::new("token", "test_token").finish();
/// let request = actix_web::test::TestRequest::delete()
///     .uri("localhost/api/deauth")
///     .cookie(cookie)
///     .to_request();
/// ```
/// 
/// # Response
/// ## Ok
/// ## Error
/// - Unauthorized
#[delete("/api/deauth")]
pub async fn deauth_token(req: HttpRequest, app_state: Data<AppState>) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::Unauthorized().finish(); }
    let token = token.unwrap().value().to_string();
    
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    if Token::find(&mut redis_conn, &token).is_err() {
        return HttpResponse::Unauthorized().finish();
    }
    Token::delete(&mut redis_conn, &token);

    let mut cookie = Cookie::build("token", "0").finish();
    cookie.make_removal();
    let mut response = HttpResponse::Ok().finish();

    let _asd = HttpResponse::add_removal_cookie(&mut response, &cookie);

    response
}

/// Pipe for refreshing a token for a server specified duration
/// - url: `{domain}/api/refresh`
/// 
/// # HTTP request requirements
/// ## header
/// - cookie named `token` containing login token
/// 
/// # Example
/// ```
/// let cookie = CookieBuilder::new("token", "test_token").finish();
/// let request = actix_web::test::TestRequest::put()
///     .uri("localhost/api/refresh")
///     .cookie(cookie)
///     .to_request();
/// ```
/// 
/// # Response
/// ## Ok
/// - set cookie header containing refreshed login cookie
/// ## Error
/// - Unauthorized
#[put("/api/refresh")]
pub async fn refresh_token(req: HttpRequest, app_state: Data<AppState>) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::Unauthorized().finish(); }
    let token = token.unwrap().value().to_string();
    
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    if Token::find(&mut redis_conn, &token).is_err() {
        return HttpResponse::Unauthorized().finish();
    }
    Token::refresh(&mut redis_conn, &token);

    let cookie = Cookie::build("token", token)
    .path("/")
        .expires(Expiration::DateTime(OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + 180).unwrap()))
        .finish();

    HttpResponse::Ok().cookie(cookie).finish()
}