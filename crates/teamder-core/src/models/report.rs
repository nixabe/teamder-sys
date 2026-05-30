use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportEntityType {
    User,
    Project,
    StudyGroup,
    ProjectUpdate,
    StudyNote,
    PeerReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Pending,
    Reviewing,
    Resolved,
    Dismissed,
}

/// A user-submitted report flagging content or another user for moderation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    #[serde(rename = "_id")]
    pub id: String,
    pub reporter_id: String,
    pub entity_type: ReportEntityType,
    pub entity_id: String,
    pub reason: String,
    pub details: Option<String>,
    pub status: ReportStatus,
    pub created_at: DateTime<Utc>,
}

impl Report {
    pub fn new(
        reporter_id: impl Into<String>,
        entity_type: ReportEntityType,
        entity_id: impl Into<String>,
        reason: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            reporter_id: reporter_id.into(),
            entity_type,
            entity_id: entity_id.into(),
            reason: reason.into(),
            details,
            status: ReportStatus::Pending,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub entity_type: ReportEntityType,
    pub entity_id: String,
    pub reason: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub id: String,
    pub reporter_id: String,
    pub entity_type: ReportEntityType,
    pub entity_id: String,
    pub reason: String,
    pub details: Option<String>,
    pub status: ReportStatus,
    pub created_at: DateTime<Utc>,
}

impl From<Report> for ReportResponse {
    fn from(r: Report) -> Self {
        Self {
            id: r.id,
            reporter_id: r.reporter_id,
            entity_type: r.entity_type,
            entity_id: r.entity_id,
            reason: r.reason,
            details: r.details,
            status: r.status,
            created_at: r.created_at,
        }
    }
}
