pub mod token;
pub mod error;

use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web::dev::ServiceRequest;
use redis::Commands;

pub async fn bearer_auth_validator(req: ServiceRequest, creds: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)>{
    let client = redis::Client::open("redis://127.0.0.1/");
    if client.is_err(){
        return Err((actix_web::error::ErrorInternalServerError("database error"), req));
    }
    let conn = client.unwrap().get_connection();
    if conn.is_err() {
        return Err((actix_web::error::ErrorInternalServerError("database error"), req));
    }
    let mut conn = conn.unwrap();

    let config = req.app_data::<Config>().map(|data| data.as_ref().clone()).unwrap_or_else(Default::default);
    println!("Auth");
    
    match conn.get::<&str, String>(creds.token()) {
        Ok(res) => {
            Ok(req)
        },
        Err(err) => {
            Err((actix_web::error::ErrorBadRequest("error"), req))
        }
    }
}