use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Sub-structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRole {
    pub name: String,
    #[serde(default)]
    pub count_needed: u8,
    #[serde(default)]
    pub filled: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: String,
    #[serde(default)]
    pub initials: String,
    #[serde(default)]
    pub color: String,
    pub joined_at: DateTime<Utc>,
    #[serde(default)]
    pub role: Option<String>,
}

// ── Main document ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: String,

    pub name: String,
    pub lead_user_id: String,

    #[serde(default = "default_project_icon")]
    pub icon: String,

    #[serde(default)]
    pub icon_bg: String,

    #[serde(default = "default_recruiting")]
    pub status: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub goals: Option<String>,

    #[serde(default)]
    pub roles: Vec<ProjectRole>,

    #[serde(default)]
    pub skills: Vec<String>,

    #[serde(default)]
    pub team: Vec<TeamMember>,

    #[serde(default)]
    pub deadline: Option<String>,

    #[serde(default)]
    pub collab: Option<String>,

    #[serde(default)]
    pub duration: Option<String>,

    #[serde(default)]
    pub category: Option<String>,

    #[serde(default = "default_true")]
    pub is_public: bool,

    #[serde(default = "default_join_mode")]
    pub join_mode: String,

    #[serde(default)]
    pub is_promoted: bool,

    #[serde(default)]
    pub banner_image: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_project_icon() -> String {
    "Pr".to_string()
}

fn default_recruiting() -> String {
    "recruiting".to_string()
}

fn default_true() -> bool {
    true
}

fn default_join_mode() -> String {
    "direct".to_string()
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub goals: Option<String>,
    #[serde(default)]
    pub roles: Option<Vec<ProjectRole>>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub collab: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub join_mode: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_bg: Option<String>,
    #[serde(default)]
    pub banner_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProjectRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub goals: Option<String>,
    #[serde(default)]
    pub roles: Option<Vec<ProjectRole>>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub collab: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub join_mode: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_bg: Option<String>,
    #[serde(default)]
    pub is_promoted: Option<bool>,
    #[serde(default)]
    pub banner_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub lead_user_id: String,
    pub icon: String,
    pub icon_bg: String,
    pub status: String,
    pub description: String,
    pub goals: Option<String>,
    pub roles: Vec<ProjectRole>,
    pub skills: Vec<String>,
    pub team: Vec<TeamMember>,
    pub deadline: Option<String>,
    pub collab: Option<String>,
    pub duration: Option<String>,
    pub category: Option<String>,
    pub is_public: bool,
    pub join_mode: String,
    pub is_promoted: bool,
    pub banner_image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            name: p.name,
            lead_user_id: p.lead_user_id,
            icon: p.icon,
            icon_bg: p.icon_bg,
            status: p.status,
            description: p.description,
            goals: p.goals,
            roles: p.roles,
            skills: p.skills,
            team: p.team,
            deadline: p.deadline,
            collab: p.collab,
            duration: p.duration,
            category: p.category,
            is_public: p.is_public,
            join_mode: p.join_mode,
            is_promoted: p.is_promoted,
            banner_image: p.banner_image,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
