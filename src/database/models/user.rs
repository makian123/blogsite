use diesel::{
    PgConnection,
    prelude::*,
    r2d2::{
        PooledConnection,
        ConnectionManager
    }
};
use crate::{
    schema::{
        users,
        self
    },
    app::AppError,
    database::models::blog::Blog
};

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
    pub fn new <'a>(conn: &PooledConnection<ConnectionManager<PgConnection>>, uname: &String, pw: &String, admin: bool) -> Result<User, &'a str>{
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

    /** Deletes an user, also deletes anything related to the user (files in the blogs included)
     */
    pub fn delete(&self, conn: &PooledConnection<ConnectionManager<PgConnection>>) {
        use schema::users::*;

        let the_id = self.id.clone();
        let blogs = Blog::get_by_creator_id(conn, &self.id);
        for blog in blogs {
            Blog::delete_by_id(conn, blog.id);
        }
        let _result = diesel::delete(users::table).filter(id.eq(the_id)).execute(conn);
    }

    /** Returns an user with the id specified */
    pub fn find_by_id(conn: &PooledConnection<ConnectionManager<PgConnection>>, user_id: &String) -> Result<User, AppError>{
        use crate::schema::users::dsl::*;
    
        let user_found = users.filter(id.eq(user_id)).load::<User>(conn);
    
        match user_found {
            Ok(ret) => {
                if ret.len() == 0 {
                    return Err(AppError::BadRequest);
                }
    
                Ok(ret[0].clone())
            },
            Err(_msg) => Err(AppError::UnauthorizedError)
        }
    }

    /// Returns first option of `User` type found with the specified username.
    /// If no user is found, or an error occurs a `None` option will be returned.
    /// # Example
    /// ```
    /// let user_found = find_user_by_username(&conn, &"username".to_string());
    /// match user_found{
    ///     Some(user) => {
    ///         println!("{:?}", user);
    ///     },
    ///     None => {
    ///         println!("No user found");
    ///     }
    /// }
    /// ```
    pub fn find_user_by_username(conn: &PooledConnection<ConnectionManager<PgConnection>>, uname: &String) -> Option<User> {
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
