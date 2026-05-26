use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Sub-structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: String,
    #[serde(default)]
    pub initials: String,
    #[serde(default)]
    pub color: String,
    #[serde(with = "crate::serde_helpers::flexible_datetime")]
    pub joined_at: DateTime<Utc>,
    #[serde(default, with = "crate::serde_helpers::flexible_datetime_opt")]
    pub last_checkin: Option<DateTime<Utc>>,
    #[serde(default)]
    pub streak: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyNote {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub title: String,
    pub body: String,
    #[serde(with = "crate::serde_helpers::flexible_datetime")]
    pub created_at: DateTime<Utc>,
}

// ── Main document ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyGroup {
    #[serde(rename = "_id")]
    pub id: String,

    pub name: String,

    #[serde(default)]
    pub goal: String,

    #[serde(default = "default_sg_icon")]
    pub icon: String,

    #[serde(default)]
    pub icon_bg: String,

    #[serde(default = "default_subject")]
    pub subject: String,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub members: Vec<GroupMember>,

    #[serde(default = "default_max_members")]
    pub max_members: u8,

    #[serde(default)]
    pub schedule: String,

    #[serde(default)]
    pub duration_weeks: u8,

    #[serde(default = "default_one")]
    pub current_week: u8,

    #[serde(default = "default_true")]
    pub is_open: bool,

    #[serde(default = "default_active")]
    pub status: String,

    #[serde(default = "default_join_mode")]
    pub join_mode: String,

    #[serde(default)]
    pub banner_image: Option<String>,

    #[serde(default)]
    pub notes: Vec<StudyNote>,

    #[serde(default)]
    pub description: Option<String>,

    pub created_by: String,
    #[serde(with = "crate::serde_helpers::flexible_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::serde_helpers::flexible_datetime")]
    pub updated_at: DateTime<Utc>,
}

fn default_sg_icon() -> String {
    "Sg".to_string()
}

fn default_subject() -> String {
    "General".to_string()
}

fn default_max_members() -> u8 {
    6
}

fn default_one() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

fn default_active() -> String {
    "active".to_string()
}

fn default_join_mode() -> String {
    "direct".to_string()
}
