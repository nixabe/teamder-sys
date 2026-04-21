use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub to_user_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

impl Message {
    pub fn new(
        from_user_id: impl Into<String>,
        from_user_name: impl Into<String>,
        to_user_id: impl Into<String>,
        to_user_name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id: from_user_id.into(),
            from_user_name: from_user_name.into(),
            to_user_id: to_user_id.into(),
            to_user_name: to_user_name.into(),
            content: content.into(),
            created_at: Utc::now(),
            read: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

impl From<Message> for MessageResponse {
    fn from(m: Message) -> Self {
        Self {
            id: m.id,
            from_user_id: m.from_user_id,
            from_user_name: m.from_user_name,
            to_user_id: m.to_user_id,
            content: m.content,
            created_at: m.created_at,
            read: m.read,
        }
    }
}

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
