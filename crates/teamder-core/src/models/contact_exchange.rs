use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContactExchangeStatus {
    Pending,
    Accepted,
    Declined,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactExchange {
    #[serde(rename = "_id")]
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub status: ContactExchangeStatus,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ContactExchange {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id: from.to_string(),
            to_user_id: to.to_string(),
            status: ContactExchangeStatus::Pending,
            expires_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn accept_expiry() -> DateTime<Utc> {
        Utc::now() + Duration::days(7)
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| e < Utc::now()).unwrap_or(false)
    }
}

#[derive(Debug, Serialize)]
pub struct ContactExchangeResponse {
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub to_user_name: String,
    pub status: ContactExchangeStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactExchangeBody {
    pub to_user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RespondContactExchangeBody {
    pub accept: bool,
}
