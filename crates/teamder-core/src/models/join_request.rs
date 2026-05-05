use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JoinRequestStatus {
    Pending,
    Accepted,
    Declined,
}

/// Stored in the `join_requests` collection. Represents an application to join
/// a project, study group, or competition team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    #[serde(rename = "_id")]
    pub id: String,
    pub from_user_id: String,
    /// "project" | "study_group" | "competition_team"
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    /// owner of the entity — used to check who can respond
    pub owner_id: String,
    pub message: Option<String>,
    pub status: JoinRequestStatus,
    /// Why the user wants to join — required for projects/teams.
    #[serde(default)]
    pub motivation: Option<String>,
    /// Which role they're applying for (must match one of the project's roles).
    #[serde(default)]
    pub role_wanted: Option<String>,
    /// Hours per week they can commit.
    #[serde(default)]
    pub hours_per_week: Option<String>,
    /// Optional portfolio URL the applicant wants to highlight.
    #[serde(default)]
    pub portfolio_url: Option<String>,
    /// Free-form text describing past relevant experience.
    #[serde(default)]
    pub relevant_experience: Option<String>,
    /// When the applicant can start (RFC3339 date or free-form string).
    #[serde(default)]
    pub availability_start: Option<String>,
    /// Whether the applicant is willing/able to meet in person.
    #[serde(default)]
    pub can_meet_in_person: Option<bool>,
    /// Additional links — extra portfolio entries, write-ups, demos, etc.
    #[serde(default)]
    pub additional_links: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl JoinRequest {
    pub fn new(
        from_user_id: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        entity_name: impl Into<String>,
        owner_id: impl Into<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_user_id: from_user_id.into(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            entity_name: entity_name.into(),
            owner_id: owner_id.into(),
            message,
            status: JoinRequestStatus::Pending,
            motivation: None,
            role_wanted: None,
            hours_per_week: None,
            portfolio_url: None,
            relevant_experience: None,
            availability_start: None,
            can_meet_in_person: None,
            additional_links: vec![],
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateJoinRequestBody {
    pub entity_type: String,
    pub entity_id: String,
    pub message: Option<String>,
    #[serde(default)]
    pub motivation: Option<String>,
    #[serde(default)]
    pub role_wanted: Option<String>,
    #[serde(default)]
    pub hours_per_week: Option<String>,
    #[serde(default)]
    pub portfolio_url: Option<String>,
    #[serde(default)]
    pub relevant_experience: Option<String>,
    #[serde(default)]
    pub availability_start: Option<String>,
    #[serde(default)]
    pub can_meet_in_person: Option<bool>,
    #[serde(default)]
    pub additional_links: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RespondJoinRequestBody {
    pub accept: bool,
    /// Optional decision note from the owner.
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JoinRequestResponse {
    pub id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub owner_id: String,
    pub message: Option<String>,
    pub status: JoinRequestStatus,
    pub motivation: Option<String>,
    pub role_wanted: Option<String>,
    pub hours_per_week: Option<String>,
    pub portfolio_url: Option<String>,
    pub relevant_experience: Option<String>,
    pub availability_start: Option<String>,
    pub can_meet_in_person: Option<bool>,
    pub additional_links: Vec<String>,
    pub created_at: DateTime<Utc>,
}
