use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyGroupAnnouncement {
    #[serde(rename = "_id")]
    pub id: String,
    pub group_id: String,
    pub author_id: String,
    pub author_name: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
}

impl StudyGroupAnnouncement {
    pub fn new(
        group_id: impl Into<String>,
        author_id: impl Into<String>,
        author_name: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            group_id: group_id.into(),
            author_id: author_id.into(),
            author_name: author_name.into(),
            title: title.into(),
            content: content.into(),
            pinned: false,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnouncementBody {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub pinned: bool,
}
