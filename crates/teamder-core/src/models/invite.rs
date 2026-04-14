use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InviteStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
}

/// An invite sent from one user to another to join a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    #[serde(rename = "_id")]
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub message: Option<String>,
    pub status: InviteStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Invite {
    pub fn new(
        from_user_id: impl Into<String>,
        from_user_name: impl Into<String>,
        to_user_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id: from_user_id.into(),
            from_user_name: from_user_name.into(),
            to_user_id: to_user_id.into(),
            project_id: None,
            project_name: None,
            message: None,
            status: InviteStatus::Pending,
            created_at: now,
            expires_at: now + chrono::Duration::days(7),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SendInviteRequest {
    pub to_user_id: String,
    pub project_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RespondInviteRequest {
    pub accept: bool,
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub message: Option<String>,
    pub status: InviteStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<Invite> for InviteResponse {
    fn from(i: Invite) -> Self {
        Self {
            id: i.id,
            from_user_id: i.from_user_id,
            from_user_name: i.from_user_name,
            to_user_id: i.to_user_id,
            project_id: i.project_id,
            project_name: i.project_name,
            message: i.message,
            status: i.status,
            created_at: i.created_at,
            expires_at: i.expires_at,
        }
    }
}
