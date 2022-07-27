use diesel::r2d2::PooledConnection;
use r2d2_redis::{RedisConnectionManager, redis::{Commands, RedisError}};
use rand::distributions::{Alphanumeric, DistString};

pub struct Token{}

impl Token {
    pub fn new(redis_conn: &mut PooledConnection<RedisConnectionManager>, user_id: &String) -> String{
        let str = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);

        let _res = redis_conn.set_ex::<&String, &String, i32>(&str, user_id, 180);

        str
    }

    pub fn delete(redis_conn: &mut PooledConnection<RedisConnectionManager>, token: &String){
        match redis_conn.get::<String, String>(token.clone()) {
            Ok(_) => {
                let _res = redis_conn.del::<String, i32>(token.clone());
            },
            Err(_) => return,
        }
    }

    pub fn find(redis_conn: &mut PooledConnection<RedisConnectionManager>, token: &String) -> Result<String, RedisError>{
        redis_conn.get::<&String, String>(token)
    }

    pub fn refresh(redis_conn: &mut PooledConnection<RedisConnectionManager>, token: &String) -> bool {
        let token = Token::find(redis_conn, token);
        if token.is_err() { return false; }
        let token = token.unwrap();

        let res = redis_conn.expire::<&String, i32>(&token, 180);
        if res.is_err() { return false; }

        return true;
    }
}