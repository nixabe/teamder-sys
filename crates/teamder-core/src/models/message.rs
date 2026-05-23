use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,

    pub from_user_id: String,
    pub to_user_id: String,
    pub content: String,

    #[serde(default)]
    pub read: bool,

    pub created_at: DateTime<Utc>,
}
