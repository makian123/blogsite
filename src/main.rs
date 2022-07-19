#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
mod database;

use actix_web::{HttpServer, App, get, post, Responder, HttpResponse, HttpRequest};
use database::db_utils::*;
use database::blogs::{
    self,
    get_post_by_creator_id,
    Blog,
    new_post
};
use crate::database::users::{
    find_user_by_id, new_user
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Dummy_Blog{
    pub creator_id: i32,
    pub title: String,
    pub body: String
}

#[derive(Deserialize)]
struct Dummy_User{
    pub username: String,
    pub password: String
}

#[post("/create_user")]
async fn create_new_user(req_body: String) -> impl Responder{
    let user = serde_json::from_str::<Dummy_User>(&req_body);
    if user.is_err() {
        return HttpResponse::BadRequest();
    }
    let user = user.unwrap();
    let conn = connect_to_db();

    let final_user = new_user(&conn, &user.username, &user.password);
    println!("{:?}", final_user);

    HttpResponse::Ok()
}
#[post("/create_blog")]
async fn create_new_blog(req_body: String) -> impl Responder{
    let blog = serde_json::from_str::<Dummy_Blog>(&req_body);
    if blog.is_err() {
        return HttpResponse::BadRequest();
    }
    let blog = blog.unwrap();
    let conn = connect_to_db();

    let user = find_user_by_id(&conn, blog.creator_id);

    if user.is_none(){
        return HttpResponse::BadRequest();
    }
    let user = user.unwrap();

    let the_blog = new_post(&conn, &user, &blog.title, &blog.body);
    println!("{:?}", the_blog);

    HttpResponse::Ok()
}
#[get("/blogs/{user_id}")]
async fn get_blogs_by_id(req: HttpRequest) -> impl Responder {
    let conn = connect_to_db();

    let posts = get_post_by_creator_id(&conn, req.match_info().query("user_id").parse().unwrap());
    HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    println!("Server running...");
    HttpServer::new(|| {
        App::new()
        .service(create_new_user)
        .service(create_new_blog)
        .service(get_blogs_by_id)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}