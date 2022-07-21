use redis::Commands;

pub struct Token{
    pub val: String
}

impl Token {
    pub fn new() -> String{
        let client = redis::Client::open("redis://127.0.0.1/");
        if client.is_err(){
            return "".to_string();
        }
        let conn = client.unwrap().get_connection();
        if conn.is_err() {
            return "".to_string();
        }
        let mut conn = conn.unwrap();

        let str = String::from("asd123");

        conn.set::<String, &str, String>(str.clone(), "alive");

        str
    }

    pub fn delete(token: String){
        let client = redis::Client::open("redis://127.0.0.1/");
        if client.is_err(){
            return;
        }
        let conn = client.unwrap().get_connection();
        if conn.is_err() {
            return;
        }
        let mut conn = conn.unwrap();
        
        match conn.get::<String, String>(token) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
}