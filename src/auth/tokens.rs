use actix_web::http::header::HeaderMap;
use chrono::Utc;
use jsonwebtoken::{Header, encode, EncodingKey, decode, DecodingKey, Validation};
use jwt_simple::{prelude::*, claims};
use serde_json::{Serializer, Deserializer};
use super::errors::Error;

#[derive(Serialize, Deserialize)]
pub struct CustomHeader {
    pub is_admin: bool,
    pub user_id: String
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    user_id: String,
    privilege: bool,
    exp: usize
}

const JWT_SECRET: &[u8] = b"test_secret";

pub struct Authenticator {}

impl Authenticator {
    pub fn create_token(header: &CustomHeader) -> Result<String, Error>{
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(2))
            .expect("invalid timestamp")
            .timestamp();

        let claims = Claims {
            user_id: header.user_id.clone(),
            privilege: header.is_admin,
            exp: expiration as usize
        };

        let header = Header::new(jsonwebtoken::Algorithm::HS256);
        encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
            .map_err(|_| Error::JWTTokenCreationError)
    }

    pub async fn authorize <'a>(privileged: bool, id: Option<String>, token: &'a str) -> bool {
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::new(jsonwebtoken::Algorithm::HS256)
        ) {
            Ok(c) => {
                if privileged == c.claims.privilege && privileged == false { 
                    if id.is_none() {
                        return false;
                    }

                    return id.unwrap() == c.claims.user_id;
                }
                else if c.claims.privilege == true {
                    return true;
                }

                return false;
            },
            Err(err) => {
                println!("{:?}", err);
                return false;
            }
        };
    }
}