use std::sync::Arc;
use diesel::{r2d2::{Pool, ConnectionManager}, PgConnection};
use r2d2_redis::RedisConnectionManager;

pub struct AppState{
    pub psql_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    pub redis_pool: Arc<Pool<RedisConnectionManager>>
}

impl Clone for AppState{
    fn clone(&self) -> Self {
        Self { psql_pool: self.psql_pool.clone(), redis_pool: self.redis_pool.clone() }
    }
}
