use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration},
    delete, put,
    web::Data,
    HttpRequest, HttpResponse, Responder,
};
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
pub async fn deauth_token(req: HttpRequest, app_state: Data<AppState>) -> impl Responder {
    let token = req.cookie("token");
    if token.is_none() {
        return HttpResponse::Unauthorized().finish();
    }
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
pub async fn refresh_token(req: HttpRequest, app_state: Data<AppState>) -> impl Responder {
    let token = req.cookie("token");
    if token.is_none() {
        return HttpResponse::Unauthorized().finish();
    }
    let token = token.unwrap().value().to_string();

    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    if Token::find(&mut redis_conn, &token).is_err() {
        return HttpResponse::Unauthorized().finish();
    }
    Token::refresh(&mut redis_conn, &token);

    let cookie = Cookie::build("token", token)
        .path("/")
        .expires(Expiration::DateTime(
            OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + 180).unwrap(),
        ))
        .finish();

    HttpResponse::Ok().cookie(cookie).finish()
}

#[cfg(test)]
mod tests {
    use actix_web::{
        cookie::CookieBuilder,
        test::{self, call_service},
        App,
    };

    use super::*;

    #[actix_rt::test]
    async fn test_deauth() {
        let app_state = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_state.clone()))
                .service(super::deauth_token),
        )
        .await;

        let user_id = "123456677899".to_string();
        let token = Token::new(&mut app_state.redis_pool.get().unwrap(), &user_id);
        debug_assert!(Token::find(&mut app_state.redis_pool.get().unwrap(), &token).is_ok());

        let cookie = CookieBuilder::new("token", &token).finish();
        let req = test::TestRequest::delete()
            .uri("/api/deauth")
            .insert_header(actix_web::http::header::ContentType::json())
            .app_data(Data::new(app_state.clone()))
            .cookie(cookie)
            .to_request();

        let resp = call_service(&app, req).await;
        debug_assert!(resp.status().is_success());
        debug_assert!(Token::find(&mut app_state.redis_pool.get().unwrap(), &token).is_err())
    }

    #[actix_rt::test]
    async fn test_refresh() {
        let app_state = AppState::new(None);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_state.clone()))
                .service(super::refresh_token),
        )
        .await;

        let user_id = "123456677899".to_string();
        let token = Token::new(&mut app_state.redis_pool.get().unwrap(), &user_id);
        debug_assert!(Token::find(&mut app_state.redis_pool.get().unwrap(), &token).is_ok());

        let cookie = CookieBuilder::new("token", &token).finish();
        let req = test::TestRequest::put()
            .uri("/api/refresh")
            .insert_header(actix_web::http::header::ContentType::json())
            .app_data(Data::new(app_state.clone()))
            .cookie(cookie)
            .to_request();

        let resp = call_service(&app, req).await;
        debug_assert!(resp.status().is_success());
        debug_assert!(Token::find(&mut app_state.redis_pool.get().unwrap(), &token).is_ok());
        Token::delete(&mut app_state.redis_pool.get().unwrap(), &token)
    }
}
