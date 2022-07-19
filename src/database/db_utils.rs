use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;


/// Return a connection to the hosted database.
/// Requires a `DATABASE_URL` as a variable in enviroment.
/// 
/// # Example
/// ```
/// let database_connection = connect_to_db();
/// ```
pub fn connect_to_db() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("Enviroment vraible: 'DATABASE_URL' not set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}