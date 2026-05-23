use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Sub-structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewScores {
    pub skill: u8,         // 1-5
    pub communication: u8, // 1-5
    pub reliability: u8,   // 1-5
    pub teamwork: u8,      // 1-5
}

impl ReviewScores {
    /// Average of the four score dimensions.
    pub fn average(&self) -> f32 {
        (self.skill as f32
            + self.communication as f32
            + self.reliability as f32
            + self.teamwork as f32)
            / 4.0
    }
}

// ── Main document ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReview {
    #[serde(rename = "_id")]
    pub id: String,

    pub reviewer_id: String,
    pub reviewer_name: String,
    pub reviewee_id: String,

    #[serde(default)]
    pub project_id: Option<String>,

    #[serde(default)]
    pub study_group_id: Option<String>,

    #[serde(default)]
    pub project_name: String,

    pub scores: ReviewScores,

    #[serde(default)]
    pub body: String,

    #[serde(default)]
    pub endorsed_skills: Vec<String>,

    pub created_at: DateTime<Utc>,
}
