use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Persisted skill category. The `_id` is the human-readable key like
/// "frontend" so URLs and tag references stay stable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSkillCategory {
    #[serde(rename = "_id")]
    pub key: String,
    pub label: String,
    pub label_zh: String,
    pub order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StoredSkillCategory {
    pub fn new(key: impl Into<String>, label: impl Into<String>, label_zh: impl Into<String>, order: i32) -> Self {
        let now = Utc::now();
        Self {
            key: key.into(),
            label: label.into(),
            label_zh: label_zh.into(),
            order,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Persisted skill tag. `_id` is a UUID so duplicates / renames don't collide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSkillTag {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub name_zh: String,
    pub category_key: String,
    pub order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StoredSkillTag {
    pub fn new(name: impl Into<String>, name_zh: impl Into<String>, category_key: impl Into<String>, order: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            name_zh: name_zh.into(),
            category_key: category_key.into(),
            order,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub key: String,
    pub label: String,
    pub label_zh: String,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub label: Option<String>,
    pub label_zh: Option<String>,
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub name_zh: String,
    pub category_key: String,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub name_zh: Option<String>,
    pub category_key: Option<String>,
    pub order: Option<i32>,
    pub is_active: Option<bool>,
}
