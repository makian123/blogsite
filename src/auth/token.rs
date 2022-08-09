use diesel::r2d2::PooledConnection;
use r2d2_redis::{
    redis::{Commands, RedisError},
    RedisConnectionManager,
};
use rand::distributions::{Alphanumeric, DistString};

pub struct Token {}

impl Token {
    /** Generates a new token of aphanumeric type and length of 32 characters. Automatically inserts it into the redis database specified*/
    pub fn new(
        redis_conn: &mut PooledConnection<RedisConnectionManager>,
        user_id: &String,
    ) -> String {
        let mut str = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
        let mut iters = 0;
        while Token::find(redis_conn, &str).is_ok() {
            str = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
            if iters > 200 {
                return "".to_string();
            }
            iters += 1;
        }

        let _res = redis_conn.set_ex::<&String, &String, i32>(&str, user_id, 180);

        str
    }

    /** Deletes a token from the database. If the token does not exist, it does nothing
     */
    pub fn delete(redis_conn: &mut PooledConnection<RedisConnectionManager>, token: &String) {
        match redis_conn.get::<String, String>(token.clone()) {
            Ok(_) => {
                let _res = redis_conn.del::<String, i32>(token.clone());
            }
            Err(_) => return,
        }
    }

    /** Returns `user_id` if found, and if not returns an error */
    pub fn find(
        redis_conn: &mut PooledConnection<RedisConnectionManager>,
        token: &String,
    ) -> Result<String, RedisError> {
        redis_conn.get::<&String, String>(token)
    }

    /** Refreshes the token for 180 seconds if token is found
     * If the token is not found or refreshed it returns `false`, if it's successfully refreshed it returns `true`
     */
    pub fn refresh(
        redis_conn: &mut PooledConnection<RedisConnectionManager>,
        token: &String,
    ) -> bool {
        let token = Token::find(redis_conn, token);
        if token.is_err() {
            return false;
        }
        let token = token.unwrap();

        let res = redis_conn.expire::<&String, i32>(&token, 180);
        if res.is_err() {
            return false;
        }

        return true;
    }
}
