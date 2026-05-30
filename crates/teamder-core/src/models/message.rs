use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_kind() -> String { "text".to_string() }

/// Stored in the `messages` collection. Only IDs — names resolved at API layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
    /// "text" | "contact_request" | "contact_accepted" | "contact_declined"
    #[serde(default = "default_kind")]
    pub kind: String,
}

impl Message {
    pub fn new(
        from_user_id: impl Into<String>,
        to_user_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id: from_user_id.into(),
            to_user_id: to_user_id.into(),
            content: content.into(),
            created_at: Utc::now(),
            read: false,
            kind: "text".to_string(),
        }
    }

    pub fn system(
        from_user_id: impl Into<String>,
        to_user_id: impl Into<String>,
        content: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        let mut m = Self::new(from_user_id, to_user_id, content);
        m.kind = kind.into();
        m
    }
}

/// Response DTO — from_user_name resolved at API layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
    #[serde(default = "default_kind")]
    pub kind: String,
}

/// Conversation summary — partner_name resolved at API layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub partner_id: String,
    pub partner_name: String,
    pub last_message: String,
    pub last_at: DateTime<Utc>,
    pub unread_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct WsIncoming {
    pub to_user_id: String,
    pub content: String,
}
