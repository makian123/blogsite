use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

/// Return a connection to the hosted database.
/// Requires a `DATABASE_URL` as a variable in enviroment.
/// 
/// # Example
/// ```
/// let database_connection = psql_connect_to_db();
/// ```
pub fn psql_connect_to_db() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("Enviroment vraible: 'DATABASE_URL' not set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

/// Return a connection to the hosted database.
/// 
/// # Example
/// ```
/// let database_connection = redis_connect_to_db();
/// ```
pub fn redis_connect_to_db() -> redis::Connection {
    dotenv().ok();

    redis::Client::open("redis://127.0.0.1/").expect("Error connecting to redis").get_connection().unwrap()
}