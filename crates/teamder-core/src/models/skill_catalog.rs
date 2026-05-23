use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Persisted skill category (MongoDB collection: skill_categories).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSkillCategory {
    /// Key string, e.g. "frontend", "backend".
    #[serde(rename = "_id")]
    pub id: String,

    pub label: String,
    pub label_zh: String,
    pub order: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Persisted skill tag (MongoDB collection: skill_tags).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSkillTag {
    #[serde(rename = "_id")]
    pub id: String, // UUID

    pub name: String,
    pub name_zh: String,
    pub category_key: String,
    pub order: i32,

    #[serde(default = "default_true")]
    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_true() -> bool {
    true
}
