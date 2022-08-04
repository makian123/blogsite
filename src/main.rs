#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate redis;

pub mod schema;
pub mod database;
pub mod app;

mod auth;
mod routes;

use actix_web::{HttpServer, App};
use routes::{user::*, blog::*, token::*, comment::*};
use app::AppState;
use crate::database::db_utils::{redis_connect_to_db, psql_connect_to_db};

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let postgres_pool = psql_connect_to_db(None);
    let redis_pool = redis_connect_to_db(None);

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