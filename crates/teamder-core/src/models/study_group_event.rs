use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyGroupEvent {
    #[serde(rename(serialize = "id", deserialize = "_id"))]
    pub id: String,
    pub group_id: String,
    pub author_id: String,
    pub author_name: String,
    pub title: String,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub description: Option<String>,
    /// user_ids of members who have RSVP'd
    #[serde(default)]
    pub attendees: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl StudyGroupEvent {
    pub fn new(
        group_id: impl Into<String>,
        author_id: impl Into<String>,
        author_name: impl Into<String>,
        title: impl Into<String>,
        location: impl Into<String>,
        starts_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            group_id: group_id.into(),
            author_id: author_id.into(),
            author_name: author_name.into(),
            title: title.into(),
            location: location.into(),
            starts_at,
            ends_at: None,
            description: None,
            attendees: vec![],
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateEventBody {
    pub title: String,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub description: Option<String>,
}
