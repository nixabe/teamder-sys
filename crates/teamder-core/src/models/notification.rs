use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    Invite,
    InviteAccepted,
    InviteDeclined,
    JoinRequest,
    JoinAccepted,
    JoinDeclined,
    Review,
    Message,
    CompetitionRecommend,
    System,
}

/// A single in-app notification targeted at one user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    #[serde(rename = "_id")]
    pub id: String,
    pub user_id: String,
    pub kind: NotificationKind,
    pub title: String,
    pub body: String,
    /// Optional internal link the frontend can navigate to (e.g. "/profile/abc").
    pub link: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

impl Notification {
    pub fn new(
        user_id: impl Into<String>,
        kind: NotificationKind,
        title: impl Into<String>,
        body: impl Into<String>,
        link: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            kind,
            title: title.into(),
            body: body.into(),
            link,
            read: false,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub kind: NotificationKind,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Notification> for NotificationResponse {
    fn from(n: Notification) -> Self {
        Self {
            id: n.id,
            kind: n.kind,
            title: n.title,
            body: n.body,
            link: n.link,
            read: n.read,
            created_at: n.created_at,
        }
    }
}
