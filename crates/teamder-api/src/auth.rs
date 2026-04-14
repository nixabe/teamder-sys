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

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-jwt-secret";

    #[test]
    fn test_create_token_returns_non_empty_string() {
        let token = create_token("user-123", "user@test.com", false, SECRET).unwrap();
        assert!(!token.is_empty());
        // JWT has three dot-separated parts
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_verify_token_round_trip() {
        let token = create_token("user-abc", "alice@test.com", false, SECRET).unwrap();
        let claims = verify_token(&token, SECRET).unwrap();
        assert_eq!(claims.sub, "user-abc");
        assert_eq!(claims.email, "alice@test.com");
        assert!(!claims.is_admin);
    }

    #[test]
    fn test_verify_token_admin_flag_preserved() {
        let token = create_token("admin-1", "admin@test.com", true, SECRET).unwrap();
        let claims = verify_token(&token, SECRET).unwrap();
        assert!(claims.is_admin);
    }

    #[test]
    fn test_verify_token_wrong_secret_returns_unauthorized() {
        let token = create_token("user-1", "u@test.com", false, SECRET).unwrap();
        let result = verify_token(&token, "wrong-secret");
        assert!(matches!(result, Err(TeamderError::Unauthorized)));
    }

    #[test]
    fn test_verify_token_invalid_string_returns_unauthorized() {
        let result = verify_token("not.a.valid.jwt.token", SECRET);
        assert!(matches!(result, Err(TeamderError::Unauthorized)));
    }

    #[test]
    fn test_verify_token_empty_string_returns_unauthorized() {
        let result = verify_token("", SECRET);
        assert!(matches!(result, Err(TeamderError::Unauthorized)));
    }

    #[test]
    fn test_token_exp_is_in_future() {
        let token = create_token("user-1", "u@test.com", false, SECRET).unwrap();
        let claims = verify_token(&token, SECRET).unwrap();
        let now = Utc::now().timestamp();
        assert!(claims.exp > now);
    }

    #[test]
    fn test_token_iat_is_in_past_or_now() {
        let token = create_token("user-1", "u@test.com", false, SECRET).unwrap();
        let claims = verify_token(&token, SECRET).unwrap();
        let now = Utc::now().timestamp();
        assert!(claims.iat <= now);
    }
}
