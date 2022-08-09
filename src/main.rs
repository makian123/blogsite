#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate redis;

pub mod app;
pub mod database;
pub mod schema;

mod auth;
mod routes;

use actix_web::{App, HttpServer};
use app::AppState;
use routes::{blog::*, comment::*, token::*, user::*};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = AppState::new(None);

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
