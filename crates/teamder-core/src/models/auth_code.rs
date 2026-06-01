use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A short-lived verification code for passwordless auth, stored in the
/// `auth_codes` collection. One active code per (email, purpose); requesting a
/// new code replaces any prior one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCode {
    #[serde(rename = "_id")]
    pub id: String,
    /// Normalised (trimmed, lower-cased) email the code was issued to.
    pub email: String,
    /// 6-digit numeric code.
    pub code: String,
    /// What the code authorises: "register", "login", or "delete".
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AuthCode {
    pub fn new(email: impl Into<String>, code: impl Into<String>, purpose: impl Into<String>, ttl_minutes: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.into(),
            code: code.into(),
            purpose: purpose.into(),
            expires_at: now + chrono::Duration::minutes(ttl_minutes),
            created_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }
}
