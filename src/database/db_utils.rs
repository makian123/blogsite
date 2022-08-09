use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use r2d2_redis::RedisConnectionManager;
use std::env;
use std::sync::Arc;

/// Return a pool to the hosted postgresql database.
/// Requires a `DATABASE_URL` as a variable in enviroment.
/// It takes argument `Option<u32>` as a number of max connections, Some(x) for x connections and None for unlimited
///
/// # Example
/// ```
/// let database_connection: Arc<Pool<ConnectionManager<PgConnection>>> = psql_connect_to_db(None);
/// ```
pub fn psql_connect_to_db(max_size: Option<u32>) -> Arc<Pool<ConnectionManager<PgConnection>>> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("Enviroment variable: 'DATABASE_URL' not set");

    let psql_manager = ConnectionManager::<diesel::PgConnection>::new(database_url);
    match max_size {
        Some(size) => Arc::new(Pool::builder().max_size(size).build(psql_manager).unwrap()),
        None => Arc::new(Pool::new(psql_manager).unwrap()),
    }
}

/// Return a pool to the hosted redis database.
/// Requires a `REDIS_DATABASE_URL` as a variable in enviroment.
/// It takes argument `Option<u32>` as a number of max connections, Some(x) for x connections and None for unlimited
/// # Example
/// ```
/// let database_connection: Arc<Pool<RedisConnectionManager>> = redis_connect_to_db(None);
/// ```
pub fn redis_connect_to_db(max_size: Option<u32>) -> Arc<Pool<RedisConnectionManager>> {
    dotenv().ok();
    let redis_url =
        env::var("REDIS_DATABASE_URL").expect("Enviroment var 'REDIS_DATABASE_URL' not set");

    let redis_manager = r2d2_redis::RedisConnectionManager::new(redis_url).unwrap();

    match max_size {
        Some(size) => Arc::new(Pool::builder().max_size(size).build(redis_manager).unwrap()),
        None => Arc::new(Pool::new(redis_manager).unwrap()),
    }
}
