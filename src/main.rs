#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate redis;

pub mod schema;
pub mod database;
pub mod app;

mod auth;
mod infrastructure;

use std::{sync::Arc};
use actix_web::{HttpServer, App};
use diesel::r2d2::{self, ConnectionManager};
use infrastructure::{user::*, blog::*, token::*, comment::*};
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
        //User routes
        .service(login)
        .service(create_new_user)
        .service(delete_an_user)
        //Blog routes
        .service(create_new_blog)
        .service(edit_blogs)
        .service(like_a_blog)
        .service(get_blogs_by_user)
        .service(create_new_blog)
        .service(delete_blog)
        .service(get_image)
        //Comment routes
        .service(create_comment)
        .service(get_comments)
        .service(delete_comment)
        //Token routes
        .service(deauth_token)
        .service(refresh_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}