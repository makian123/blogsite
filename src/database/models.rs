use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use crate::schema::{users, blogs, self};
use chrono::{NaiveDateTime, Utc};

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

#[derive(Debug)]
#[derive(Queryable)]
#[derive(Clone)]
pub struct User{
    pub id: String,
    pub username: String,
    ///SHA256 of the password
    pub pass: String,
    pub is_admin: bool
}

#[derive(Insertable)]
#[table_name="users"]
pub struct UserInsert{
    pub username: String,
    pub pass: String,
    pub is_admin: bool
}

impl User {
    /// Pushes a new user object in the database and returns a result
    /// of `User` or `&str`
    /// 
    /// # Example
    /// ```
    /// let result = new_user(
    ///     &conn, 
    ///     "username".to_string(), 
    ///     "SHA256 of the password".to_string());
    /// ```
    pub fn new <'a>(conn: &PgConnection, uname: &String, pw: &String, admin: bool) -> Result<User, &'a str>{
        if pw.len() != 64 {
            return Err("Invalid password hash length");
        }

        if uname.len() == 0 {
            return Err("Username not specified");
        }

        let to_insert = UserInsert {
            username: uname.clone(),
            pass: pw.clone(),
            is_admin: admin
        };

        let ret_user: User = diesel::insert_into(schema::users::table)
            .values(&to_insert)
            .get_result(conn)
            .expect("Error");

        Ok(ret_user)
    }

    pub fn delete(&self, conn: &PgConnection) {
        use schema::users::*;
        let the_id = self.id.clone();
        let _result = diesel::delete(users::table).filter(id.eq(the_id)).execute(conn);
    }

    pub fn find_by_id(conn: &PgConnection, user_id: &String) -> Option<User>{
        use crate::schema::users::dsl::*;
    
        let user_found = users.filter(id.eq(user_id)).load::<User>(conn);
    
        match user_found {
            Ok(ret) => {
                if ret.len() == 0 {
                    return None;
                }
    
                Some(ret[0].clone())
            },
            Err(_msg) => None
        }
    }

    /// Returns first option of `User` type found with the specified username.
    /// If no user is found, or an error occurs a `None` option will be returned.
    /// # Example
    /// ```
    /// let user find_user_by_username(&conn, &"username".to_string());
    /// match user{
    ///     Some(usr) => {
    ///         println!("{:?}", usr);
    ///     },
    ///     None => {
    ///         println!("No user found");
    ///     }
    /// }
    /// ```
    pub fn find_user_by_username(conn: &PgConnection, uname: &String) -> Option<User> {
        use crate::schema::users::dsl::*;

        let user_found = users.filter(username.eq(uname)).load::<User>(conn);

        match user_found {
            Ok(ret) => {
                if ret.len() == 0 {
                    return None;
                }

                Some(ret[0].clone())
            },
            Err(_msg) => None
        }
    }

}