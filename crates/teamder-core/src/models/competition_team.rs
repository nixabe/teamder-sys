use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Sub-structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompTeamMember {
    pub user_id: String,
    pub name: String,
    #[serde(default)]
    pub initials: String,
    #[serde(default)]
    pub role: Option<String>,
    pub joined_at: DateTime<Utc>,
}

// ── Main document ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionTeam {
    #[serde(rename = "_id")]
    pub id: String,

    pub competition_id: String,
    pub competition_name: String,
    pub name: String,

    #[serde(default)]
    pub description: String,

    pub lead_user_id: String,

    #[serde(default)]
    pub members: Vec<CompTeamMember>,

    #[serde(default = "default_max_members")]
    pub max_members: u8,

    #[serde(default)]
    pub looking_for: Vec<String>,

    #[serde(default)]
    pub open_roles: Vec<String>,

    #[serde(default = "default_recruiting")]
    pub status: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_max_members() -> u8 {
    5
}

fn default_recruiting() -> String {
    "recruiting".to_string()
}
