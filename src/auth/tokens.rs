use jwt_simple::prelude::*;
use serde_json::{Serializer, Deserializer};

#[derive(Serialize, Deserialize)]
pub struct CustomHeader {
    pub is_admin: bool,
    pub user_id: i32
}

pub struct Authenticator {
    key: HS256Key
}

impl Authenticator {
    pub fn is_valid_token <'a>(&self, token: &'a str) -> Result<CustomHeader, jwt_simple::Error> {
        let claim = self.key.verify_token::<CustomHeader>(&token, None)?;
    }
}