use std::time::{SystemTime, UNIX_EPOCH};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use crate::schema::{blogs, self};
use serde::{Serialize, Deserialize};
use crate::database::users::User;

pub fn new_post <'a>(conn: &PgConnection, creator: &User, title: &String, body: &String) -> Result<Blog, &'a str>{
    if title.len() == 0 {
        return Err("No title found");
    }
    if body.len() == 0 {
        return Err("No body found");
    }

    let time: i64 = SystemTime::now().duration_since(UNIX_EPOCH).expect("Couldn't generate time").as_secs() as i64;

    let to_insert = BlogInsert {
        title: title.clone(),
        body: body.clone(),
        creator_id: creator.id,
        created_time: time,
        last_edited_time: time,
        likes: 0
    };

    let ret_blog: Blog = diesel::insert_into(schema::blogs::table)
        .values(&to_insert)
        .get_result(conn)
        .expect("Error");

    Ok(ret_blog)
}

pub fn get_posts(conn: &PgConnection) -> Vec<Blog>{
    use crate::schema::blogs::dsl::*;

    let users = blogs.load::<Blog>(conn);

    if users.is_err() {
        return Vec::new();
    }

    users.unwrap()
}

pub fn get_post_by_creator_id(conn: &PgConnection, creator: i32) -> Vec<Blog> {
    use crate::schema::blogs::dsl::*;

    let user_blogs = blogs.filter(creator_id.eq(creator)).load::<Blog>(conn);
    if user_blogs.is_err() {
        return Vec::new();
    }

    user_blogs.unwrap()
}

#[derive(Insertable)]
#[table_name="blogs"]
struct BlogInsert {
    pub title: String,
    pub body: String,
    pub creator_id: i32,
    pub created_time: i64,
    pub last_edited_time: i64,
    pub likes: i32
}

#[derive(Debug)]
#[derive(Queryable)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct Blog {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub creator_id: i32,
    pub created_time: i64,
    pub last_edited_time: i64,
    pub likes: i32
}