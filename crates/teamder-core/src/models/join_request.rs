use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Main document ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    #[serde(rename = "_id")]
    pub id: String,

    pub from_user_id: String,

    /// "project" | "study_group" | "competition_team"
    pub entity_type: String,

    pub entity_id: String,

    #[serde(default)]
    pub entity_name: String,

    pub owner_id: String,

    #[serde(default)]
    pub message: Option<String>,

    #[serde(default = "default_pending")]
    pub status: String,

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

    #[serde(default)]
    pub comm_channels: Vec<String>,

    #[serde(default)]
    pub timezone: Option<String>,

    #[serde(default)]
    pub agreed_to_coc: bool,

    #[serde(default)]
    pub skill_confidence: Vec<String>,

    pub created_at: DateTime<Utc>,
}

fn default_pending() -> String {
    "pending".to_string()
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// The rich join-request form body from the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJoinRequestBody {
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default)]
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
    pub additional_links: Option<Vec<String>>,
    #[serde(default)]
    pub comm_channels: Option<Vec<String>>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub agreed_to_coc: Option<bool>,
    #[serde(default)]
    pub skill_confidence: Option<Vec<String>>,
}

/// Response DTO (same shape but includes derived fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequestResponse {
    pub id: String,
    pub from_user_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub owner_id: String,
    pub message: Option<String>,
    pub status: String,
    pub motivation: Option<String>,
    pub role_wanted: Option<String>,
    pub hours_per_week: Option<String>,
    pub portfolio_url: Option<String>,
    pub relevant_experience: Option<String>,
    pub availability_start: Option<String>,
    pub can_meet_in_person: Option<bool>,
    pub additional_links: Vec<String>,
    pub comm_channels: Vec<String>,
    pub timezone: Option<String>,
    pub agreed_to_coc: bool,
    pub skill_confidence: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl From<JoinRequest> for JoinRequestResponse {
    fn from(jr: JoinRequest) -> Self {
        Self {
            id: jr.id,
            from_user_id: jr.from_user_id,
            entity_type: jr.entity_type,
            entity_id: jr.entity_id,
            entity_name: jr.entity_name,
            owner_id: jr.owner_id,
            message: jr.message,
            status: jr.status,
            motivation: jr.motivation,
            role_wanted: jr.role_wanted,
            hours_per_week: jr.hours_per_week,
            portfolio_url: jr.portfolio_url,
            relevant_experience: jr.relevant_experience,
            availability_start: jr.availability_start,
            can_meet_in_person: jr.can_meet_in_person,
            additional_links: jr.additional_links,
            comm_channels: jr.comm_channels,
            timezone: jr.timezone,
            agreed_to_coc: jr.agreed_to_coc,
            skill_confidence: jr.skill_confidence,
            created_at: jr.created_at,
        }
    }
}
