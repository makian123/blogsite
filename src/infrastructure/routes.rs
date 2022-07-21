use actix_web::{get, post, Responder, HttpResponse, HttpRequest, delete};
use crate::database::db_utils::*;
use crate::database::models::{User, Blog};
use serde::{Deserialize};
use serde_json::Value;
use sha256::digest;

#[derive(Deserialize)]
struct DummyBlog{
    pub title: String,
    pub body: String
}

#[derive(Deserialize)]
struct DummyUser{
    pub username: String,
    pub password: String
}

//User routes
#[get("/login")]
async fn login(_req: HttpRequest, req_body: String) -> impl Responder{
    let credentials: Value = serde_json::from_str(&req_body).unwrap();
    if credentials.get("username").is_none() || credentials.get("password").is_none(){
        return HttpResponse::BadRequest().body("Bad request");
    }
    
    let conn = connect_to_db();
    let username = credentials.get("username").unwrap().as_str().unwrap().to_string();
    let pw = digest(credentials.get("password").unwrap().as_str().unwrap().to_string());

    let user = User::find_user_by_username(&conn, &username);
    if user.is_none() {
        return HttpResponse::BadRequest().body("No user found");
    }
    let user = user.unwrap();

    if user.pass != pw {
        return HttpResponse::BadRequest().body("Invalid credentials");
    }

    HttpResponse::Accepted().body(user.id)
}
#[post("/create_user")]
async fn create_new_user(req_body: String) -> impl Responder{
    let user = serde_json::from_str::<DummyUser>(&req_body);
    if user.is_err() {
        return HttpResponse::BadRequest();
    }
    let user = user.unwrap();
    let conn = connect_to_db();

    if User::find_user_by_username(&conn, &user.username).is_some() {
        return HttpResponse::BadRequest();
    }

    let final_user = User::new(&conn, &user.username, &digest(user.password), true);
    println!("{:?}", final_user);

    HttpResponse::Ok()
}
#[delete("/users/{username}")]
async fn delete_an_user(req: HttpRequest) -> impl Responder {
    let user_id = req.headers().get("user_id");
    let username = req.match_info().query("username").to_string();
    if user_id.is_none() {
        return HttpResponse::BadRequest();
    }
    let user_id = String::from(user_id.unwrap().to_str().unwrap());

    let conn = connect_to_db();
    let user = User::find_by_id(&conn, &user_id);
    if user.is_none() {
        return HttpResponse::BadRequest();
    }
    let user = user.unwrap();
    if user.username != username {
        return HttpResponse::BadRequest();
    }

    Blog::delete_by_user_id(&conn, &user.id);
    user.delete(&conn);

    HttpResponse::Ok()
}

//Blog routes
#[post("/create_blog")]
async fn create_new_blog(req: HttpRequest, req_body: String) -> impl Responder{
    if req.headers().get("user_id").is_none() {
        println!("Header val missing");
        return HttpResponse::BadRequest();
    }

    let blog = serde_json::from_str::<DummyBlog>(&req_body);
    if blog.is_err() {
        println!("Blog info missing");
        return HttpResponse::BadRequest();
    }
    let blog = blog.unwrap();
    let conn = connect_to_db();

    let user_id = req.headers().get("user_id").unwrap().to_str();
    if user_id.is_err() {
        println!("User id missing");
        return HttpResponse::BadRequest();
    }
    let user_id = String::from(user_id.unwrap().to_string());

    /*if Authenticator::authorize(
            false, 
            Some(user_id), 
            req.headers().get("token").unwrap().to_str().unwrap()
            ).await == false {
        println!("Authentication failed");
        return HttpResponse::BadRequest();
    }*/

    let user = User::find_by_id(&conn, &user_id);
    if user.is_none(){
        println!("User found");
        return HttpResponse::BadRequest();
    }
    let user = user.unwrap();

    if Blog::new(&conn, &user, &blog.title, &blog.body).is_err() {
        return HttpResponse::InternalServerError();
    }

    HttpResponse::Ok()
}
#[get("/blogs/{username}")]
async fn get_blogs_by_id(req: HttpRequest) -> impl Responder {
    let username = req.match_info().query("username").to_string();

    let conn = connect_to_db();
    let user = User::find_user_by_username(&conn, &username);
    if user.is_none() {
        return HttpResponse::BadRequest().body("");
    }
    let user = user.unwrap();

    let posts = Blog::get_by_creator_id(&conn, &user.id);
    HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap())
}
#[post("/blogs/{blog_id}/edit")]
async fn edit_blogs(req: HttpRequest, req_body: String) -> impl Responder {
    if req.headers().get("user_id").is_none() {
        return HttpResponse::BadRequest();
    }
    
    //Checks for request body, if there's none, throw bad request
    let updated_blog = serde_json::from_str(&req_body);
    let user_id = String::from(req.headers().get("user_id").unwrap().to_str().unwrap());
    if updated_blog.is_err() {
        return HttpResponse::BadRequest();
    }
    let updated_blog: Value = updated_blog.unwrap();

    //Starts a db and tries to find user from supplied id
    //if no user found, bad request
    let conn = connect_to_db();
    let usr = User::find_by_id(&conn, &user_id);
    let blog_id = req.match_info().query("blog_id").to_string();
    if usr.is_none() {
        return HttpResponse::BadRequest();
    }
    let usr = usr.unwrap();

    //Tries to find a blog posted by that user with the id
    //if no blog found throw bad request
    let mut blogs = Blog::get_by_creator_id(&conn, &usr.id);
    let blog = blogs.iter_mut().find(|x| x.id.to_string() == blog_id);
    if blog.is_none() {
        return HttpResponse::BadRequest();
    }
    let blog = blog.unwrap();

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
            Some(_x) => {Some(&title_optional)},
            None => {None}
        },
        match body {
            Some(_x) => {Some(&body_optional)},
            None => {None}
        },match likes {
            Some(_x) => {Some(likes_optional)},
            None => {None}
        },
    );

    HttpResponse::Ok()
}
