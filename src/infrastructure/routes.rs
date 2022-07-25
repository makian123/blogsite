use actix_web::cookie::time::OffsetDateTime;
use actix_web::cookie::{Cookie, Expiration};
use actix_web::{get, post, Responder, HttpResponse, HttpRequest, delete};
use chrono::Utc;
use crate::database::db_utils::*;
use crate::database::models::{User, Blog};
use serde::{Deserialize};
use serde_json::Value;
use sha256::digest;
use crate::auth::token::Token;

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
    
    let psql_conn = psql_connect_to_db();
    let mut redis_conn = redis_connect_to_db();
    let username = credentials.get("username").unwrap().as_str().unwrap().to_string();
    let pw = digest(credentials.get("password").unwrap().as_str().unwrap().to_string());

    let user = User::find_user_by_username(&psql_conn, &username);
    if user.is_none() {
        return HttpResponse::Unauthorized().finish();
    }
    let user = user.unwrap();

    if user.pass != pw {
        return HttpResponse::Unauthorized().finish();
    }

    let token = Token::new(&mut redis_conn, &user.id);
    let cookie = Cookie::build("token", token)
        .path("/")
        .expires(Expiration::DateTime(OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + 180).unwrap()))
        .finish();

    HttpResponse::Ok().cookie(cookie).finish()
}
#[post("/create_user")]
async fn create_new_user(req_body: String) -> impl Responder{
    let user = serde_json::from_str::<DummyUser>(&req_body);
    if user.is_err() { return HttpResponse::BadRequest(); }
    let user = user.unwrap();
    let conn = psql_connect_to_db();

    if User::find_user_by_username(&conn, &user.username).is_some() { return HttpResponse::BadRequest(); }

    let _final_user = User::new(&conn, &user.username, &digest(user.password), true);

    HttpResponse::Ok()
}
#[delete("/users/{username}")]
async fn delete_an_user(req: HttpRequest) -> impl Responder {
    let username = req.match_info().query("username").to_string();
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest(); }
    let token = token.unwrap().value().to_string();
    let mut redis_conn = redis_connect_to_db();
    let user_id = Token::find(&mut redis_conn, &token);
    if user_id.is_err() { return HttpResponse::BadRequest(); }
    let user_id = user_id.unwrap();

    let conn = psql_connect_to_db();
    let user = User::find_by_id(&conn, &user_id);
    if user.is_none() { return HttpResponse::BadRequest(); }
    let user = user.unwrap();
    if user.username != username { return HttpResponse::BadRequest(); }
    Blog::delete_by_user_id(&conn, &user.id);
    Token::delete(&mut redis_conn, &token);
    user.delete(&conn);

    HttpResponse::Ok()
}

//Blog routes
#[post("/create_blog")]
async fn create_new_blog(req: HttpRequest, req_body: String) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest(); }
    let token = token.unwrap().value().to_string();

    let blog = serde_json::from_str::<DummyBlog>(&req_body);
    if blog.is_err() { return HttpResponse::BadRequest(); }
    let blog = blog.unwrap();
    let psql_conn = psql_connect_to_db();
    let mut redis_conn = redis_connect_to_db();

    let user_id = Token::find(&mut redis_conn, &token);
    if user_id.is_err() { return HttpResponse::BadRequest(); }
    let user_id = String::from(user_id.unwrap().to_string());

    let user = User::find_by_id(&psql_conn, &user_id);
    if user.is_none(){ return HttpResponse::BadRequest(); }
    let user = user.unwrap();

    if Blog::new(&psql_conn, &user, &blog.title, &blog.body).is_err() { return HttpResponse::InternalServerError(); }

    HttpResponse::Ok()
}
#[get("/blogs/{username}")]
async fn get_blogs_by_id(req: HttpRequest) -> impl Responder {
    let username = req.match_info().query("username").to_string();

    let conn = psql_connect_to_db();
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
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest(); }
    let token = token.unwrap().value().to_string();
    
    //Checks for request body, if there's none, throw bad request
    let updated_blog = serde_json::from_str(&req_body);
    if updated_blog.is_err() { return HttpResponse::BadRequest(); }
    let updated_blog: Value = updated_blog.unwrap();

    //Starts a db and tries to find user from supplied id
    //if no user found, bad request
    let mut redis_conn = redis_connect_to_db();
    let psql_conn = psql_connect_to_db();

    let user_id = Token::find(&mut redis_conn, &token);
    if user_id.is_err() { return HttpResponse::BadRequest(); }
    let user_id = user_id.unwrap();
    let usr = User::find_by_id(&psql_conn, &user_id);
    let blog_id = req.match_info().query("blog_id").to_string();
    if usr.is_none() { return HttpResponse::BadRequest(); }
    let usr = usr.unwrap();

    //Tries to find a blog posted by that user with the id
    //if no blog found throw bad request
    let mut blogs = Blog::get_by_creator_id(&psql_conn, &usr.id);
    let blog = blogs.iter_mut().find(|x| x.id.to_string() == blog_id);
    if blog.is_none() { return HttpResponse::BadRequest(); }
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

    blog.edit(&psql_conn, 
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
#[post("/blogs/{blog_id}/like")]
async fn like_a_blog(req: HttpRequest) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest().finish(); }
    let token = token.unwrap().value().to_string();

    let psql_conn = psql_connect_to_db();
    let mut redis_conn = redis_connect_to_db();
    let blog_id = req.match_info().query("blog_id").parse().unwrap();

    if Token::find(&mut redis_conn, &token).is_err() { return HttpResponse::BadRequest().finish(); }

    let blog = Blog::get_by_id(&psql_conn, blog_id);
    if blog.is_none() {
        return HttpResponse::BadRequest().finish();
    }
    let mut blog = blog.unwrap();
    blog.edit(&psql_conn, None, None, Some(blog.likes + 1));

    HttpResponse::Ok().finish()
}

//Token things
#[delete("/api/deauth")]
async fn deauth_token(req: HttpRequest) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest().finish(); }
    let token = token.unwrap().value().to_string();
    
    let mut redis_conn = redis_connect_to_db();
    if Token::find(&mut redis_conn, &token).is_err() {
        return HttpResponse::BadRequest().finish();
    }
    Token::delete(&mut redis_conn, &token);

    let mut cookie = Cookie::build("token", "0").finish();
    cookie.make_removal();
    let mut response = HttpResponse::Ok().finish();

    let _asd = HttpResponse::add_removal_cookie(&mut response, &cookie);

    response
}
#[post("/api/refresh")]
async fn refresh_token(req: HttpRequest) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest().finish(); }
    let token = token.unwrap().value().to_string();
    
    let mut redis_conn = redis_connect_to_db();
    if Token::find(&mut redis_conn, &token).is_err() {
        return HttpResponse::BadRequest().finish();
    }
    Token::refresh(&mut redis_conn, &token);

    let cookie = Cookie::build("token", token)
    .path("/")
        .expires(Expiration::DateTime(OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + 180).unwrap()))
        .finish();

    HttpResponse::Ok().cookie(cookie).finish()
}