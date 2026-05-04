use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A timeline entry posted by a project lead — used to broadcast progress,
/// announcements, milestones, or asks for help.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectUpdateKind {
    Progress,
    Milestone,
    Announcement,
    HelpWanted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUpdate {
    #[serde(rename = "_id")]
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub author_name: String,
    pub kind: ProjectUpdateKind,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl ProjectUpdate {
    pub fn new(
        project_id: impl Into<String>,
        author_id: impl Into<String>,
        author_name: impl Into<String>,
        kind: ProjectUpdateKind,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            author_id: author_id.into(),
            author_name: author_name.into(),
            kind,
            title: title.into(),
            body: body.into(),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectUpdateRequest {
    pub kind: ProjectUpdateKind,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectUpdateResponse {
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub author_name: String,
    pub kind: ProjectUpdateKind,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl From<ProjectUpdate> for ProjectUpdateResponse {
    fn from(u: ProjectUpdate) -> Self {
        Self {
            id: u.id,
            project_id: u.project_id,
            author_id: u.author_id,
            author_name: u.author_name,
            kind: u.kind,
            title: u.title,
            body: u.body,
            created_at: u.created_at,
        }
    }
}
