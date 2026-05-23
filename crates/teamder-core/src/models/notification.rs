use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    #[serde(rename = "_id")]
    pub id: String,

    pub user_id: String,

    /// invite | invite_accepted | invite_declined | join_request | join_accepted
    /// | join_declined | review | message | competition_recommend | system
    pub kind: String,

    pub title: String,

    #[serde(default)]
    pub body: String,

    #[serde(default)]
    pub link: Option<String>,

    #[serde(default)]
    pub read: bool,

    pub created_at: DateTime<Utc>,
}
