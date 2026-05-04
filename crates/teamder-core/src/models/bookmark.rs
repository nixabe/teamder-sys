use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BookmarkKind {
    User,
    Project,
    Competition,
    StudyGroup,
    CompetitionTeam,
}

/// A saved item — letting users keep a personal shortlist across the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(rename = "_id")]
    pub id: String,
    pub user_id: String,
    pub kind: BookmarkKind,
    pub entity_id: String,
    /// Cached display name to avoid extra lookups on the bookmarks page.
    pub label: String,
    pub created_at: DateTime<Utc>,
}

impl Bookmark {
    pub fn new(
        user_id: impl Into<String>,
        kind: BookmarkKind,
        entity_id: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            kind,
            entity_id: entity_id.into(),
            label: label.into(),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmarkRequest {
    pub kind: BookmarkKind,
    pub entity_id: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct BookmarkResponse {
    pub id: String,
    pub kind: BookmarkKind,
    pub entity_id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

impl From<Bookmark> for BookmarkResponse {
    fn from(b: Bookmark) -> Self {
        Self { id: b.id, kind: b.kind, entity_id: b.entity_id, label: b.label, created_at: b.created_at }
    }
}
