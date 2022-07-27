use actix_web::{get, post, put, HttpRequest, web::Data, Responder, HttpResponse, delete};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, auth::token::Token, database::models::{User, Blog, Like}};

#[derive(Deserialize)]
struct DummyBlog{
    pub title: String,
    pub body: String
}

//Blog routes
#[post("/blog")]
pub async fn create_new_blog(req: HttpRequest, req_body: String, app_state: Data<AppState>) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest(); }
    let token = token.unwrap().value().to_string();

    let blog = serde_json::from_str::<DummyBlog>(&req_body);
    if blog.is_err() { return HttpResponse::BadRequest(); }
    let blog = blog.unwrap();
    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

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
pub async fn get_blogs_by_id(req: HttpRequest, app_state: Data<AppState>) -> impl Responder {
    let username = req.match_info().query("username").to_string();

    let conn = app_state.psql_pool.clone().get().unwrap();
    let user = User::find_user_by_username(&conn, &username);
    if user.is_none() {
        return HttpResponse::BadRequest().body("");
    }
    let user = user.unwrap();

    let posts = Blog::get_by_creator_id(&conn, &user.id);
    HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap())
}
#[put("/blogs/{blog_id}")]
pub async fn edit_blogs(req: HttpRequest, req_body: String, app_state: Data<AppState>) -> impl Responder {
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest(); }
    let token = token.unwrap().value().to_string();
    
    //Checks for request body, if there's none, throw bad request
    let updated_blog = serde_json::from_str(&req_body);
    if updated_blog.is_err() { return HttpResponse::BadRequest(); }
    let updated_blog: Value = updated_blog.unwrap();

    //Starts a db and tries to find user from supplied id
    //if no user found, bad request
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let psql_conn = app_state.psql_pool.clone().get().unwrap();

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

    if title.is_some() {
        title_optional = title.unwrap().as_str().unwrap().to_string();
    }
    if body.is_some() {
        body_optional = body.unwrap().as_str().unwrap().to_string();
    }

    blog.edit(&psql_conn, 
        match title {
            Some(_x) => {Some(&title_optional)},
            None => {None}
        },
        match body {
            Some(_x) => {Some(&body_optional)},
            None => {None}
        },
        None
    );

    HttpResponse::Ok()
}
#[put("/blogs/{blog_id}/like")]
pub async fn like_a_blog(req: HttpRequest, app_state: Data<AppState>) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest().finish(); }
    let token = token.unwrap().value().to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let blog_id = req.match_info().query("blog_id").parse().unwrap();

    if Token::find(&mut redis_conn, &token).is_err() { return HttpResponse::BadRequest().finish(); }
    let user_id = Token::find(&mut redis_conn, &token).unwrap();

    let blog = Blog::get_by_id(&psql_conn, blog_id);
    if blog.is_none() {
        return HttpResponse::BadRequest().finish();
    }
    let mut blog = blog.unwrap();
    let like = Like::new(&psql_conn, &user_id, blog_id);

    if like.is_none() {
        blog.edit(&psql_conn, None, None, Some(blog.likes - 1));
        Like::delete(&psql_conn, &user_id, blog_id);
        return HttpResponse::Ok().finish();
    }
    blog.edit(&psql_conn, None, None, Some(blog.likes + 1));


    HttpResponse::Ok().finish()
}
#[delete("/blog/{blog_id}")]
async fn delete_blog(req: HttpRequest, app_state: Data<AppState>) -> impl Responder {
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::BadRequest().finish(); }
    let token = token.unwrap().value().to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    let user_id = Token::find(&mut redis_conn, &token);
    if user_id.is_err() { return HttpResponse::BadRequest().finish(); }
    let user_id = String::from(user_id.unwrap().to_string());
    let user = User::find_by_id(&psql_conn, &user_id);
    if user.is_none(){ return HttpResponse::BadRequest().finish(); }
    let user = user.unwrap();

    let blog_id = req.match_info().query("blog_id").parse::<i32>();
    if blog_id.is_err() { return HttpResponse::BadRequest().finish(); }
    let blog_id = blog_id.unwrap();
    let blog = Blog::get_by_id(&psql_conn, blog_id);
    if blog.is_none() { return HttpResponse::BadRequest().finish(); }
    let blog = blog.unwrap();

    if user.id != blog.created_by && !user.is_admin { return HttpResponse::Forbidden().finish(); }

    Blog::delete_by_id(&psql_conn, blog.id);

    HttpResponse::Ok().finish()
}