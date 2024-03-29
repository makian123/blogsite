use crate::schema::likes;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};

#[derive(Insertable, Queryable)]
#[table_name = "likes"]
pub struct Like {
    pub user_id: String,
    pub blog_id: i32,
}

impl Like {
    /** Creates a like with `user` and `blog` id's specified */
    pub fn new(
        conn: &PooledConnection<ConnectionManager<PgConnection>>,
        user: &String,
        blog: i32,
    ) -> Option<Like> {
        let like = Like {
            user_id: user.clone(),
            blog_id: blog.clone(),
        };
        let res = diesel::insert_into(likes::table)
            .values(&like)
            .get_result(conn);

        if res.is_err() {
            return None;
        }
        Some(res.unwrap())
    }
    /** Gets all likes by the user id */
    pub fn get_by_user_id(
        conn: &PooledConnection<ConnectionManager<PgConnection>>,
        user: String,
    ) -> Vec<Like> {
        use crate::schema::likes::dsl::*;
        let likes_found = likes.filter(user_id.eq(user)).load::<Like>(conn);
        if likes_found.is_err() {
            return Vec::new();
        }

        likes_found.unwrap()
    }
    /** Gets all likes from blog id */
    pub fn get_by_blog_id(
        conn: &PooledConnection<ConnectionManager<PgConnection>>,
        blog: i32,
    ) -> Vec<Like> {
        use crate::schema::likes::dsl::*;
        let likes_found = likes.filter(blog_id.eq(blog)).load::<Like>(conn);
        if likes_found.is_err() {
            return Vec::new();
        }

        likes_found.unwrap()
    }
    /** Deletes a like */
    pub fn delete(
        conn: &PooledConnection<ConnectionManager<PgConnection>>,
        user: &String,
        blog: i32,
    ) {
        use crate::schema::likes::dsl::*;
        let _temp =
            diesel::delete(likes.filter(user_id.eq(user)).filter(blog_id.eq(blog))).execute(conn);
    }
}
