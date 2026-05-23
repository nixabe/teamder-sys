use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(rename = "_id")]
    pub id: String,

    pub user_id: String,

    /// user | project | competition | study_group | competition_team
    pub kind: String,

    pub entity_id: String,

    #[serde(default)]
    pub label: String,

    pub created_at: DateTime<Utc>,
}
