use super::user::*;
use crate::{
    app::AppError,
    schema::{self, blogs},
};
use chrono::{NaiveDateTime, Utc};
use diesel::{prelude::*, PgConnection};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, PartialEq, PartialOrd, Eq, Queryable, Clone, Serialize, Deserialize)]
pub struct Blog {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub image_id: Option<String>,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub likes: i32,
}

#[derive(Insertable)]
#[table_name = "blogs"]
struct BlogInsert {
    pub title: String,
    pub body: String,
    pub image_id: Option<String>,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub likes: i32,
}

impl Ord for Blog {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.created_at == other.created_at {
            std::cmp::Ordering::Equal
        } else if self.created_at < other.created_at {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

impl Blog {
    /** Creates a new blog which is inserted into the database,
     * `img_id` parameter takes filename of the image
     */
    pub fn new(
        conn: &PgConnection,
        creator: &User,
        title_in: &String,
        body_in: &String,
        img_id: Option<&String>,
    ) -> Result<Blog, AppError> {
        if title_in.len() == 0 || body_in.len() == 0 {
            return Err(AppError::BadRequest);
        }

        let time = Utc::now().naive_utc();

        let to_insert = BlogInsert {
            title: title_in.clone(),
            body: body_in.clone(),
            created_by: creator.id.clone(),
            created_at: time,
            updated_at: time,
            likes: 0,
            image_id: match img_id {
                Some(img) => Some(img.clone()),
                None => None,
            },
        };

        let ret_blog = diesel::insert_into(schema::blogs::table)
            .values(&to_insert)
            .get_result(conn)?;

        Ok(ret_blog)
    }

    /** Returns all blogs which were created by the specified user,
     * vector is automatically sorted from the most recent being first to the oldest being last
     */
    pub fn get_by_creator_id(conn: &PgConnection, creator: &String) -> Vec<Blog> {
        use crate::schema::blogs::dsl::*;

        let user_blogs = blogs
            .filter(created_by.eq(creator))
            .order(created_at.desc())
            .load::<Blog>(conn);
        if user_blogs.is_err() {
            return Vec::new();
        }
        let mut user_blogs = user_blogs.unwrap();

        user_blogs.sort();

        user_blogs
    }

    /** Returns blog containing certain id */
    pub fn get_by_id(conn: &PgConnection, blog_id: i32) -> Option<Blog> {
        use crate::schema::blogs::dsl::*;

        let blog = blogs.filter(id.eq(blog_id)).load::<Blog>(conn);
        if blog.is_err() {
            return None;
        }

        Some(blog.unwrap()[0].clone())
    }

    /** Deletes all blogs created by certain user (this does not remove images used in the blogs) */
    pub fn delete_by_user_id(conn: &PgConnection, user_id: &String) {
        use crate::schema::blogs::dsl::*;

        let _result = diesel::delete(schema::blogs::table)
            .filter(created_by.eq(user_id))
            .execute(conn);
    }
    /** Deletes a blog with the specified id, also deletes the file associated with the blog */
    pub fn delete_by_id(conn: &PgConnection, blog_id_in: i32) {
        use crate::schema::blogs::dsl::*;
        use crate::schema::likes::dsl::*;

        let blog: Blog = blogs.find(blog_id_in).first::<Blog>(conn).unwrap();
        if blog.image_id.is_some() {
            let res = fs::remove_file(PathBuf::from(
                "images/".to_string() + &blog.image_id.clone().unwrap(),
            ));
            if res.is_err() {
                return;
            }
        }

        let _result = diesel::delete(schema::blogs::table)
            .filter(id.eq(blog_id_in))
            .execute(conn);
        let _result = diesel::delete(schema::likes::table)
            .filter(blog_id.eq(blog_id_in))
            .execute(conn);
    }

    /** Edits any of the given parameters of the blog */
    pub fn edit(
        &mut self,
        conn: &PgConnection,
        title_in: Option<&String>,
        body_in: Option<&String>,
        likes_in: Option<i32>,
    ) {
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
            .set((
                title.eq(&self.title),
                body.eq(&self.body),
                likes.eq(self.likes),
            ))
            .execute(conn);
    }
}
