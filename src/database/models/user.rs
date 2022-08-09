use crate::{
    app::AppError,
    database::models::blog::Blog,
    schema::{self, users},
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};

pub trait UserTrait {
    fn new(
        conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>,
        uname: &String,
        pw: &String,
        admin: bool,
    ) -> Result<User, AppError>;
    fn delete(&self, conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>);
    fn find_by_id(
        conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>,
        user_id: &String,
    ) -> Result<User, AppError>;
    fn find_by_username(
        conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>,
        uname: &String,
    ) -> Option<User>;
}

#[derive(Debug, Queryable, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    ///SHA256 of the password
    pub pass: String,
    pub is_admin: bool,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct UserInsert {
    pub username: String,
    pub pass: String,
    pub is_admin: bool,
}

impl UserTrait for User {
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
    fn new(
        conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>,
        uname: &String,
        pw: &String,
        admin: bool,
    ) -> Result<User, AppError> {
        if pw.len() != 64 || uname.len() == 0 {
            return Err(AppError::BadRequest);
        }

        let to_insert = UserInsert {
            username: uname.clone(),
            pass: pw.clone(),
            is_admin: admin,
        };

        let ret_user: User = diesel::insert_into(schema::users::table)
            .values(&to_insert)
            .get_result(conn.ok_or(AppError::InternalServerError)?)?;

        Ok(ret_user)
    }

    /** Deletes an user, also deletes anything related to the user (files in the blogs included)
     */
    fn delete(&self, conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>) {
        use schema::users::*;
        let conn = conn.unwrap();

        let the_id = self.id.clone();
        let blogs = Blog::get_by_creator_id(conn, &self.id);
        for blog in blogs {
            Blog::delete_by_id(conn, blog.id);
        }
        let _result = diesel::delete(users::table)
            .filter(id.eq(the_id))
            .execute(conn);
    }

    /** Returns an user with the id specified */
    fn find_by_id(
        conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>,
        user_id: &String,
    ) -> Result<User, AppError> {
        use crate::schema::users::dsl::*;

        let user_found = users.filter(id.eq(user_id)).load::<User>(conn.unwrap());

        match user_found {
            Ok(ret) => {
                if ret.len() == 0 {
                    return Err(AppError::BadRequest);
                }

                Ok(ret[0].clone())
            }
            Err(_msg) => Err(AppError::UnauthorizedError),
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
    fn find_by_username(
        conn: Option<&PooledConnection<ConnectionManager<PgConnection>>>,
        uname: &String,
    ) -> Option<User> {
        use crate::schema::users::dsl::*;

        let user_found = users.filter(username.eq(uname)).load::<User>(conn.unwrap());

        match user_found {
            Ok(ret) => {
                if ret.len() == 0 {
                    return None;
                }

                Some(ret[0].clone())
            }
            Err(_msg) => None,
        }
    }
}
