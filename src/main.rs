#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
mod database;

use actix_web::{HttpServer, App, get, post, Responder, HttpResponse, HttpRequest};
use database::db_utils::*;
use database::blogs::{
    get_post_by_creator_id,
    new_post,
    delete_posts_by_user_id
};
use crate::database::users::{
    find_user_by_id, new_user
};
use serde::Deserialize;
use serde_json::Value;

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
    let user_id = req.match_info().query("user_id").parse::<i32>();
    if user_id.is_err() {
        return HttpResponse::BadRequest().body("Error! No user found");
    }
    let user_id = user_id.unwrap();

    let conn = connect_to_db();

    let posts = get_post_by_creator_id(&conn, user_id);
    HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap())
}
#[post("/blogs/{user_id}/{blog_id}/edit")]
async fn edit_blogs(req: HttpRequest, req_body: String) -> impl Responder {
    //Checks for request body, if there's none, throw bad request
    let updated_blog = serde_json::from_str(&req_body);
    if updated_blog.is_err() {
        return HttpResponse::BadRequest();
    }
    let updated_blog: Value = updated_blog.unwrap();

    //Starts a db and tries to find user from supplied id
    //if no user found, bad request
    let conn = connect_to_db();
    let usr = find_user_by_id(&conn, req.match_info().query("user_id").parse().unwrap());
    if usr.is_none() {
        return HttpResponse::BadRequest();
    }
    let usr = usr.unwrap();
    let blog_id = req.match_info().query("blog_id").parse::<i32>().unwrap();

    //Tries to find a blog posted by that user with the id
    //if no blog found throw bad request
    let mut blogs = get_post_by_creator_id(&conn, usr.id);
    let mut blog = blogs.iter_mut().find(|x| x.id == blog_id);
    if blog.is_none() {
        return HttpResponse::BadRequest();
    }
    let mut blog = blog.unwrap();

    //Tries to parse the json values into normal values if they exist
    let title = updated_blog.get("title");
        let mut title_optional = String::new();
    let body = updated_blog.get("body");
        let mut body_optional = String::new();
    let likes = updated_blog.get("likes");
        let mut likes_optional = 0;

    if title.is_some() {
        title_optional = title.unwrap().as_str().unwrap().to_string();
    }
    if body.is_some() {
        body_optional = body.unwrap().as_str().unwrap().to_string();
    }
    if likes.is_some() {
        likes_optional = likes.unwrap().as_i64().unwrap() as i32;
    }

    blog.edit(&conn, 
        match title {
            Some(x) => {Some(&title_optional)},
            None => {None}
        },
        match body {
            Some(x) => {Some(&body_optional)},
            None => {None}
        },match likes {
            Some(x) => {Some(likes_optional)},
            None => {None}
        },
    );

    HttpResponse::Ok()
}
#[post("/users/{user_id}/delete")]
async fn delete_an_user(req: HttpRequest) -> impl Responder {
    let user_id = req.match_info().query("user_id").parse::<i32>();
    if user_id.is_err() {
        return HttpResponse::BadRequest();
    }
    let user_id = user_id.unwrap();

    let conn = connect_to_db();
    let user = find_user_by_id(&conn, user_id);
    if user.is_none() {
        return HttpResponse::BadRequest();
    }
    let user = user.unwrap();

    delete_posts_by_user_id(&conn, user.id);
    user.delete(&conn);

    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    println!("Server running...");
    HttpServer::new(|| {
        App::new()
        .service(create_new_user)
        .service(delete_an_user)
        .service(create_new_blog)
        .service(edit_blogs)
        .service(get_blogs_by_id)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}