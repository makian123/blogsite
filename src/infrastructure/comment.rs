use actix_web::{
    post, 
    HttpRequest, 
    web::Data, 
    Responder, 
    HttpResponse, 
    delete, get
};
use crate::{
    app::AppState,
    auth::token::Token,
    database::models::{
        user::*, 
        blog::*, 
        comment::*
    }
};

#[post("/blogs/{blog_id}/comment")]
pub async fn create_comment(req: HttpRequest, req_body: String, app_state: Data<AppState>) -> impl Responder{
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::Unauthorized().finish(); }
    let token = token.unwrap().value().to_string();
    let blog_id = req.match_info().query("blog_id").parse::<i32>().unwrap();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    if Blog::get_by_id(&psql_conn, blog_id).is_none() { return HttpResponse::BadRequest().finish(); }

    let user_id = Token::find(&mut redis_conn, &token);
    if user_id.is_err() { return HttpResponse::Unauthorized().finish(); }
    let user_id = String::from(user_id.unwrap().to_string());

    let comment = Comment::new(&psql_conn, blog_id, &user_id, &req_body);
    if comment.is_none() { return HttpResponse::BadRequest().finish(); }

    HttpResponse::Ok().finish()
}
#[get("/blogs/{blog_id}/comments")]
pub async fn get_comments(req: HttpRequest, app_state: Data<AppState>) -> impl Responder {
    let blog_id = req.match_info().query("blog_id").parse::<i32>().unwrap();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();

    let comments = Comment::find_by_blog(&psql_conn, blog_id).unwrap();

    HttpResponse::Ok().body(serde_json::to_string(&comments).unwrap())
}
#[delete("/blogs/{blog_id}/comments/{comment_id}")]
pub async fn delete_comment(req: HttpRequest, app_state: Data<AppState>) -> impl Responder {
    let token = req.cookie("token");
    if token.is_none() { return HttpResponse::Unauthorized().finish(); }
    let token = token.unwrap().value().to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    let user_id = Token::find(&mut redis_conn, &token);
    if user_id.is_err() { return HttpResponse::Unauthorized().finish(); }
    let user_id = user_id.unwrap();
    let user = User::find_by_id(&psql_conn, &user_id);
    if user.is_none() { return HttpResponse::BadRequest().finish(); }
    let user = user.unwrap();

    let comment_id = req.match_info().query("comment_id").to_string();
    let comment = Comment::find_by_id(&psql_conn, &comment_id);
    if comment.is_none() { return HttpResponse::BadRequest().finish(); }
    let comment = comment.unwrap();

    if comment.user_id != user.id && !user.is_admin { return HttpResponse::Forbidden().finish(); }

    Comment::delete(&psql_conn, &comment_id);

    HttpResponse::Ok().finish()
}