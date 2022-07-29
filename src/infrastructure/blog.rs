use std::{path::PathBuf, io::Write, fs};

use actix_multipart::{Multipart};
use actix_web::{get, post, put, HttpRequest, web::{Data, self}, HttpResponse, delete};
use serde_json::Value;
use uuid::Uuid;
use futures::{stream::StreamExt as _, TryStreamExt};
use crate::{app::{AppState, AppError}, auth::token::Token, database::models::{user::*, blog::*, like::*}};

fn extract_extension(buffer: &[u8]) -> String{
    let mut ret = String::new();
    let mut started = false;

    for i in 0..buffer.len() {
         if match buffer[i]{
            65 ..= 90 => {
                if !started {
                    started = true;
                }
                ret.push(buffer[i] as char);
                true
            },
            _ => {
                if started {
                    false
                }
                else {
                    true
                }
            }
        } == false {
            break;
        }
    }

    ret
}

async fn parse_multipart(payload: &mut Multipart) -> Result<(String, String, String), AppError> {
    let (mut title, mut body, mut filename) = (String::new(), String::new(), String::new());

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition();
        
        if content_type.get_name().is_some() {
            match content_type.get_name().unwrap() {
                "image" => {
                    let file_name = Uuid::new_v4();
                    let mut p = PathBuf::new();
                    p.push(format!("images/{}", file_name.clone().to_string()));
                    let mut cloned_path = p.clone();
                    let real_path = p.clone();
                    filename.clear();
                    let mut file = web::block(|| {std::fs::File::create(p)})
                        .await
                        .unwrap()
                        .unwrap();

                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();

                        file = web::block(move || {
                            file.write_all(&data).map(|_| file)
                        })
                        .await
                        .unwrap()
                        .unwrap();
                    }

                    let the_file = fs::read(real_path.clone()).unwrap();
                    let ret = extract_extension(&the_file[..]);
                    cloned_path.set_extension(ret);
                    let _return = fs::rename(real_path, cloned_path.clone());
                    
                    filename.push_str(&cloned_path.to_str().unwrap().to_string());
                },
                "title" => {
                    title.clear();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        title.push_str(std::str::from_utf8(&data).unwrap());
                    }
                },
                "body" => {
                    body.clear();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        body.push_str(std::str::from_utf8(&data).unwrap());
                    }
                },
                _ => {}
            };
        }
    }

    if title.len() == 0 && body.len() == 0 {
        return Err(AppError::BadRequest);
    }

    Ok((title, body, filename))
}

//Blog routes
#[post("/blog")]
pub async fn create_new_blog(req: HttpRequest, app_state: Data<AppState>, mut mp: Multipart) -> Result<HttpResponse, AppError>{
    let (title, body, filename) = parse_multipart(&mut mp).await?;
    let token = req.cookie("token").ok_or(AppError::Forbidden)?.value().to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    let user_id = Token::find(&mut redis_conn, &token)?;

    let user = User::find_by_id(&psql_conn, &user_id)?;

    Blog::new(&psql_conn, &user, &title, &body,
        if filename.len() == 0 {
            None
        }
        else {
            Some(&filename)
        }
    )?;

    Ok(HttpResponse::Ok().finish())
}
#[get("/blogs/{username}")]
pub async fn get_blogs_by_id(req: HttpRequest, app_state: Data<AppState>) -> Result<HttpResponse, AppError> {
    let username = req.match_info().query("username").to_string();

    let conn = app_state.psql_pool.clone().get().unwrap();
    let user = User::find_user_by_username(&conn, &username).ok_or(AppError::UnauthorizedError)?;

    let posts = Blog::get_by_creator_id(&conn, &user.id);
    Ok(HttpResponse::Ok().body(serde_json::to_string(&posts).unwrap()))
}
#[put("/blogs/{blog_id}")]
pub async fn edit_blogs(req: HttpRequest, req_body: String, app_state: Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = req.cookie("token").ok_or(AppError::UnauthorizedError)?.value().to_string();
    
    //Checks for request body, if there's none, throw bad request
    let updated_blog = serde_json::from_str::<Value>(&req_body).map_err(|_| AppError::BadRequest)?;

    //Starts a db and tries to find user from supplied id
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let psql_conn = app_state.psql_pool.clone().get().unwrap();

    let _user_id = Token::find(&mut redis_conn, &token)?;
    let blog_id = req.match_info().query("blog_id").parse::<i32>()?;

    //Tries to find a blog posted by that user with the id
    //if no blog found throw bad request
    let mut blog = Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;
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

    Ok(HttpResponse::Ok().finish())
}
#[put("/blogs/{blog_id}/like")]
pub async fn like_a_blog(req: HttpRequest, app_state: Data<AppState>) -> Result<HttpResponse, AppError>{
    let token = req.cookie("token").ok_or(AppError::UnauthorizedError)?.value().to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();
    let blog_id = req.match_info().query("blog_id").parse().unwrap();

    //Confirms user is logged in
    Token::find(&mut redis_conn, &token)?;
    let user_id = Token::find(&mut redis_conn, &token).unwrap();

    let mut blog = Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;

    let like = Like::new(&psql_conn, &user_id, blog_id);
    if like.is_none() {
        blog.edit(&psql_conn, None, None, Some(blog.likes - 1));
        Like::delete(&psql_conn, &user_id, blog_id);
        return Ok(HttpResponse::Ok().finish());
    }
    blog.edit(&psql_conn, None, None, Some(blog.likes + 1));


    Ok(HttpResponse::Ok().finish())
}
#[delete("/blogs/{blog_id}")]
pub async fn delete_blog(req: HttpRequest, app_state: Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = req.cookie("token").ok_or(AppError::UnauthorizedError)?.value().to_string();

    let psql_conn = app_state.psql_pool.clone().get().unwrap();
    let mut redis_conn = app_state.redis_pool.clone().get().unwrap();

    let user_id = Token::find(&mut redis_conn, &token)?;
    let user = User::find_by_id(&psql_conn, &user_id)?;

    let blog_id = req.match_info().query("blog_id").parse::<i32>()?;
    let blog = Blog::get_by_id(&psql_conn, blog_id).ok_or(AppError::BadRequest)?;

    if user.id != blog.created_by && !user.is_admin { 
        return Err(AppError::Forbidden)
    }

    Blog::delete_by_id(&psql_conn, blog.id);

    Ok(HttpResponse::Ok().finish())
}
#[get("/images/{image_name}")]
pub async fn get_image(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let img_name = req.match_info().query("image_name");

    let mut file_path = PathBuf::new();
    file_path.push("images");
    file_path.push(img_name);

    let file = fs::read(file_path)?;
    
    Ok(HttpResponse::Ok().body(file))
}