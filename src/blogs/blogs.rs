use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use crate::schema::{blogs::{*, self}, self};
use serde::{Serialize, Deserialize};
use crate::users::users::User;

#[derive(Insertable)]
#[table_name="blogs"]
struct BlogInsert {
    pub title: String,
    pub body: String,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub likes: i32
}

impl Blog {
    pub fn new <'a>(conn: &PgConnection, creator: &User, title_in: &String, body_in: &String) -> Result<Blog, &'a str>{
        if title_in.len() == 0 {
            return Err("No title found");
        }
        if body_in.len() == 0 {
            return Err("No body found");
        }
    
        let time = Utc::now().naive_utc();
    
        let to_insert = BlogInsert {
            title: title_in.clone(),
            body: body_in.clone(),
            created_by: creator.id.clone(),
            created_at: time,
            updated_at: time,
            likes: 0
        };
    
        let ret_blog: Blog = diesel::insert_into(schema::blogs::table)
            .values(&to_insert)
            .get_result(conn)
            .expect("Error");
    
        Ok(ret_blog)
    }
    
    pub fn get_by_creator_id(conn: &PgConnection, creator: &String) -> Vec<Blog> {
        use crate::schema::blogs::dsl::*;
    
        let user_blogs = blogs.filter(created_by.eq(creator)).load::<Blog>(conn);
        if user_blogs.is_err() {
            return Vec::new();
        }
    
        user_blogs.unwrap()
    }
    
    pub fn delete_by_user_id(conn: &PgConnection, user_id: &String){
        use crate::schema::blogs::dsl::*;
        use schema::blogs;
    
        let _result = diesel::delete(blogs::table).filter(created_by.eq(user_id)).execute(conn);
    }
    
    pub fn edit(&mut self, conn: &PgConnection, title_in: Option<&String>, body_in: Option<&String>, likes_in: Option<i32>){
        use self::schema::blogs::dsl::*;

        if title_in.is_none() && body_in.is_none() && likes_in.is_none() {
            return;
        }

        let title_in = title_in.unwrap_or(&self.title);
        let body_in = body_in.unwrap_or(&&self.body);
        let likes_in = likes_in.unwrap_or(self.likes);
        
        self.title = title_in.clone();
        self.body = body_in.clone();
        self.likes = likes_in;

        let _updated = diesel::update(blogs.filter(id.eq(self.id)))
            .set((title.eq(&self.title), body.eq(&self.body), likes.eq(self.likes)))
            .execute(conn);
    }
}