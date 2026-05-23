use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUpdate {
    #[serde(rename = "_id")]
    pub id: String,

    pub project_id: String,
    pub author_id: String,
    pub author_name: String,

    /// progress | milestone | announcement | help_wanted
    pub kind: String,

    pub title: String,

    #[serde(default)]
    pub body: String,

    pub created_at: DateTime<Utc>,
}
