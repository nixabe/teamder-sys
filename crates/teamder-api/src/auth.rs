use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use teamder_core::error::TeamderError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: user id
    pub sub: String,
    /// Email
    pub email: String,
    /// Is admin
    pub is_admin: bool,
    /// Issued at (unix timestamp)
    pub iat: i64,
    /// Expiry (unix timestamp)
    pub exp: i64,
}

pub fn create_token(
    user_id: &str,
    email: &str,
    is_admin: bool,
    secret: &str,
) -> Result<String, TeamderError> {
    let now = Utc::now();
    let exp = (now + Duration::days(30)).timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        is_admin,
        iat: now.timestamp(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| TeamderError::Internal(e.to_string()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, TeamderError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|_| TeamderError::Unauthorized)
}
