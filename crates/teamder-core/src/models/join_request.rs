use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JoinRequestStatus {
    Pending,
    Accepted,
    Declined,
}

/// Stored in the `join_requests` collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    #[serde(rename = "_id")]
    pub id: String,
    pub from_user_id: String,
    /// "project" or "study_group"
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    /// owner of the entity — used to check who can respond
    pub owner_id: String,
    pub message: Option<String>,
    pub status: JoinRequestStatus,
    pub created_at: DateTime<Utc>,
}

impl JoinRequest {
    pub fn new(
        from_user_id: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        entity_name: impl Into<String>,
        owner_id: impl Into<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id: from_user_id.into(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            entity_name: entity_name.into(),
            owner_id: owner_id.into(),
            message,
            status: JoinRequestStatus::Pending,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateJoinRequestBody {
    pub entity_type: String,
    pub entity_id: String,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RespondJoinRequestBody {
    pub accept: bool,
}

#[derive(Debug, Serialize)]
pub struct JoinRequestResponse {
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub owner_id: String,
    pub message: Option<String>,
    pub status: JoinRequestStatus,
    pub created_at: DateTime<Utc>,
}
