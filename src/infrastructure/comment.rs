use actix_web::{
    post, 
    HttpRequest, 
    web::Data,
    HttpResponse, 
    delete, get
};
use crate::{
    app::{AppState, AppError},
    auth::token::Token,
    database::models::{
        user::*, 
        blog::*, 
        comment::*
    }
};

#[post("/blogs/{blog_id}/comment")]
pub async fn create_comment(req: HttpRequest, req_body: String, app_state: Data<AppState>) -> Result<HttpResponse, AppError>{
    let token = req.cookie("token").ok_or(AppError::UnauthorizedError)?.value().to_string();
    let blog_id = req.match_info().query("blog_id").parse::<i32>().unwrap();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    //Checks if blog exists
    Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;

    let user_id = Token::find(&mut redis_conn, &token)?;
    Comment::new(&psql_conn, blog_id, &user_id, &req_body).ok_or(AppError::InternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}
#[get("/blogs/{blog_id}/comments")]
pub async fn get_comments(req: HttpRequest, app_state: Data<AppState>) -> Result<HttpResponse, AppError> {
    let blog_id = req.match_info().query("blog_id").parse::<i32>().unwrap();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();

    let comments = Comment::find_by_blog(&psql_conn, blog_id).unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string(&comments).unwrap()))
}
#[delete("/blogs/{blog_id}/comments/{comment_id}")]
pub async fn delete_comment(req: HttpRequest, app_state: Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = req.cookie("token").ok_or(AppError::UnauthorizedError)?.value().to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    let user_id = Token::find(&mut redis_conn, &token)?;
    let user = User::find_by_id(&psql_conn, &user_id)?;

    let comment_id = req.match_info().query("comment_id").to_string();
    let comment = Comment::find_by_id(&psql_conn, &comment_id).ok_or(AppError::BadRequest)?;

    if comment.user_id != user.id && !user.is_admin { 
        return Err(AppError::Forbidden); 
    }

    Comment::delete(&psql_conn, &comment_id);

    Ok(HttpResponse::Ok().finish())
}