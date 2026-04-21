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

/// An invite sent from one user to another to join a project or study group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    #[serde(rename = "_id")]
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    #[serde(default)]
    pub to_user_name: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    #[serde(default)]
    pub study_group_id: Option<String>,
    #[serde(default)]
    pub study_group_name: Option<String>,
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
        to_user_name: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id: from_user_id.into(),
            from_user_name: from_user_name.into(),
            to_user_id: to_user_id.into(),
            to_user_name: to_user_name.into(),
            project_id: None,
            project_name: None,
            study_group_id: None,
            study_group_name: None,
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
    pub study_group_id: Option<String>,
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
    pub to_user_name: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub study_group_id: Option<String>,
    pub study_group_name: Option<String>,
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
            to_user_name: i.to_user_name,
            project_id: i.project_id,
            project_name: i.project_name,
            study_group_id: i.study_group_id,
            study_group_name: i.study_group_name,
            message: i.message,
            status: i.status,
            created_at: i.created_at,
            expires_at: i.expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_invite() -> Invite {
        Invite::new("user-sender", "Alice Wang", "user-recipient", "Bob Chen")
    }

    #[test]
    fn test_default_status_pending() {
        let inv = make_invite();
        assert_eq!(inv.status, InviteStatus::Pending);
    }

    #[test]
    fn test_default_project_id_none() {
        let inv = make_invite();
        assert!(inv.project_id.is_none());
    }

    #[test]
    fn test_default_message_none() {
        let inv = make_invite();
        assert!(inv.message.is_none());
    }

    #[test]
    fn test_expires_at_is_seven_days_after_created() {
        let inv = make_invite();
        let diff = inv.expires_at - inv.created_at;
        // Should be ~7 days (604800 seconds), allow 1 second tolerance
        assert!(diff.num_seconds() >= 604799 && diff.num_seconds() <= 604801);
    }

    #[test]
    fn test_from_and_to_user_ids_stored() {
        let inv = make_invite();
        assert_eq!(inv.from_user_id, "user-sender");
        assert_eq!(inv.to_user_id, "user-recipient");
        assert_eq!(inv.from_user_name, "Alice Wang");
    }

    #[test]
    fn test_id_is_uuid_like() {
        let inv = make_invite();
        assert_eq!(inv.id.len(), 36);
    }

    #[test]
    fn test_response_from_invite() {
        let inv = make_invite();
        let resp = InviteResponse::from(inv.clone());
        assert_eq!(resp.id, inv.id);
        assert_eq!(resp.status, InviteStatus::Pending);
        assert_eq!(resp.from_user_id, "user-sender");
        assert_eq!(resp.to_user_id, "user-recipient");
    }
}
