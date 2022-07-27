#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate redis;

pub mod schema;
pub mod database;
pub mod app;

mod auth;
mod users;
mod blogs;
mod infrastructure;

use std::sync::Arc;

use actix_web::{HttpServer, App};
use diesel::r2d2::{self, ConnectionManager};
use infrastructure::{user::*, blog::*, token::*};
use app::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv::dotenv().ok();

    let psql = std::env::var("DATABASE_URL").expect("Enviroment vraible: 'DATABASE_URL' not set");
    let redis = std::env::var("REDIS_DATABASE_URL").expect("Enviroment vraible: 'DATABASE_URL' not set");
    let psql_manager = ConnectionManager::<diesel::PgConnection>::new(psql);
    let redis_manager = r2d2_redis::RedisConnectionManager::new(redis).unwrap();

    let postgres_pool = Arc::new(r2d2::Pool::new(psql_manager).unwrap());
    let redis_pool = Arc::new(r2d2::Pool::new(redis_manager).unwrap());

    let app_state = AppState{
        psql_pool: postgres_pool,
        redis_pool: redis_pool
    };

    println!("Server running...");
    HttpServer::new(move || {
        App::new()
        .app_data(actix_web::web::Data::new(app_state.clone()))
        .service(login)
        .service(create_new_user)
        .service(delete_an_user)
        .service(create_new_blog)
        .service(edit_blogs)
        .service(like_a_blog)
        .service(get_blogs_by_id)
        .service(create_new_blog)
        .service(delete_blog)
        .service(deauth_token)
        .service(refresh_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}