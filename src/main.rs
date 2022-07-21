#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate redis;

pub mod schema;
pub mod database;
mod auth;
mod users;
mod blogs;
mod infrastructure;

use actix_web::{HttpServer, App};
use actix_web_httpauth::middleware::HttpAuthentication;
use infrastructure::routes::*;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = redis_client.get_connection().unwrap();

    println!("Server running...");
    HttpServer::new(move || {
        App::new()
        .service(login)
        .service(create_new_user)
        .service(delete_an_user)
        .service(create_new_blog)
        .service(edit_blogs)
        .service(get_blogs_by_id)
        .service(create_new_blog)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}