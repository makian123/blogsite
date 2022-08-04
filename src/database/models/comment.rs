use chrono::NaiveDateTime;
use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection, RunQueryDsl, QueryDsl};
use serde::Serialize;
use crate::schema::{self, comments};
use crate::diesel::ExpressionMethods;

#[derive(Queryable, Clone, Serialize)]
pub struct Comment {
    pub id: String,
    pub user_id: String,
    pub blog_id: i32,
    pub created_at: NaiveDateTime,
    pub body: String
}

impl Comment {
    /** Creates a comment on the blog specified */
    pub fn new(conn: &PooledConnection<ConnectionManager<PgConnection>>, blog_id_in: i32, user_id_in: &String, comment_body: &String) -> Option<Comment> {
        let record = CommentInsert {
            user_id: user_id_in.clone(),
            blog_id: blog_id_in,
            body: comment_body.clone()
        };
        match diesel::insert_into(schema::comments::table)
            .values(&record)
            .get_results::<Comment>(conn){
                Ok(ret) => Some(ret[0].clone()),
                Err(_) => None
            }
    }

    /** Returns all comments with the user specified */
    pub fn find_by_user(conn: &PooledConnection<ConnectionManager<PgConnection>>, user_id_in: &String) -> Option<Vec<Comment>> {
        use schema::comments::dsl::*;
        match comments.filter(user_id.eq(user_id_in)).order(created_at.desc()).load::<Comment>(conn){
            Ok(ret) => Some(ret),
            Err(_) => None,
        }
    }
    /** Returns comment with the id specified */
    pub fn find_by_id(conn: &PooledConnection<ConnectionManager<PgConnection>>, the_id: &String) -> Option<Comment> {
        use schema::comments::dsl::*;

        match comments.filter(id.eq(the_id)).first::<Comment>(conn){
            Ok(ret) => Some(ret.clone()),
            Err(_) => None,
        }
    }
    /** Returns all comments posted in a blog */
    pub fn find_by_blog(conn: &PooledConnection<ConnectionManager<PgConnection>>, blog_id_in: i32) -> Option<Vec<Comment>> {
        use schema::comments::dsl::*;
        
        match comments.filter(blog_id.eq(blog_id_in)).order(created_at.desc()).load::<Comment>(conn){
            Ok(ret) => Some(ret),
            Err(_) => None,
        }
    }

    /** Deletes a comment from database */
    pub fn delete(conn: &PooledConnection<ConnectionManager<PgConnection>>, the_id: &String){
        use crate::schema::comments::dsl::*;
        let _ret = diesel::delete(schema::comments::table).filter(id.eq(the_id)).execute(conn);
    }
}

#[derive(Insertable)]
#[table_name="comments"]
struct CommentInsert {
    pub user_id: String,
    pub blog_id: i32,
    pub body: String
}