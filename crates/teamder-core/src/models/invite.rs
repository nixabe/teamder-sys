use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    #[serde(rename = "_id")]
    pub id: String,

    pub from_user_id: String,
    pub to_user_id: String,

    #[serde(default)]
    pub project_id: Option<String>,

    #[serde(default)]
    pub study_group_id: Option<String>,

    #[serde(default)]
    pub competition_team_id: Option<String>,

    #[serde(default)]
    pub message: Option<String>,

    #[serde(default = "default_pending")]
    pub status: String,

    #[serde(default)]
    pub is_read: bool,

    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

fn default_pending() -> String {
    "pending".to_string()
}
