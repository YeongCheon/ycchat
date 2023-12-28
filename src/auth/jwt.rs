use crate::util::variable::JWT_SECRET;
use jsonwebtoken::{
    decode as jwt_decode, encode, get_current_timestamp, DecodingKey, EncodingKey, Validation,
};
use serde::{Deserialize, Serialize};

use crate::models::user::UserId;

pub const ALGORITHM: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS256;
const ISS: &str = "ycchat";
const EXP_ACCESS_TOKEN: u64 = 3600; // 1 hour
const EXP_REFRESH_TOKEN: u64 = 3600 * 24 * 14; // 14 day

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: UserId,
    pub iat: u64,
    pub exp: u64,
}

pub fn generate_access_token(user_id: &UserId) -> Result<String, jsonwebtoken::errors::Error> {
    let key = EncodingKey::from_secret(JWT_SECRET.as_bytes());

    let claims = Claims {
        sub: "access_token".to_string(),
        aud: *user_id,
        iss: ISS.to_string(),
        iat: get_current_timestamp(),
        exp: get_current_timestamp() + EXP_ACCESS_TOKEN,
    };

    encode(&jsonwebtoken::Header::new(ALGORITHM), &claims, &key)
}

pub fn generate_refresh_token(user_id: &UserId) -> Result<String, jsonwebtoken::errors::Error> {
    let key = EncodingKey::from_secret(JWT_SECRET.as_bytes());

    let claims = Claims {
        sub: "refresh_token".to_string(),
        aud: *user_id,
        iss: ISS.to_string(),
        iat: get_current_timestamp(),
        exp: get_current_timestamp() + EXP_REFRESH_TOKEN,
    };

    encode(&jsonwebtoken::Header::new(ALGORITHM), &claims, &key)
}

pub fn decode(
    jwt_token: &str,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(JWT_SECRET.as_bytes());
    let mut validation = Validation::new(ALGORITHM);
    validation.validate_aud = false;

    jwt_decode::<Claims>(jwt_token.trim(), &key, &validation)
}
