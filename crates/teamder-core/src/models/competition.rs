use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Sub-structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub user_id: String,
    #[serde(default)]
    pub team_name: Option<String>,
    pub registered_at: DateTime<Utc>,
    #[serde(default)]
    pub motivation: Option<String>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub contact_email: Option<String>,
}

// ── Main document ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    #[serde(rename = "_id")]
    pub id: String,

    pub name: String,
    pub organizer: String,

    #[serde(default = "default_comp_icon")]
    pub icon: String,

    #[serde(default)]
    pub icon_bg: String,

    #[serde(default = "default_open")]
    pub status: String,

    #[serde(default)]
    pub prize: String,

    #[serde(default = "default_min_team")]
    pub team_size_min: u8,

    #[serde(default = "default_max_team")]
    pub team_size_max: u8,

    #[serde(default)]
    pub deadline: Option<String>,

    #[serde(default)]
    pub duration: String,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub is_featured: bool,

    #[serde(default)]
    pub banner_image: Option<String>,

    #[serde(default = "default_published")]
    pub publish_status: String,

    #[serde(default)]
    pub publisher_id: Option<String>,

    #[serde(default)]
    pub rejected_note: Option<String>,

    #[serde(default)]
    pub registrations: Vec<Registration>,

    #[serde(default)]
    pub interested_user_ids: Vec<String>,

    #[serde(default)]
    pub winners: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_comp_icon() -> String {
    "Cp".to_string()
}

fn default_open() -> String {
    "open".to_string()
}

fn default_min_team() -> u8 {
    2
}

fn default_max_team() -> u8 {
    5
}

fn default_published() -> String {
    "published".to_string()
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompetitionRequest {
    pub name: String,
    pub organizer: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_bg: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub prize: Option<String>,
    #[serde(default)]
    pub team_size_min: Option<u8>,
    #[serde(default)]
    pub team_size_max: Option<u8>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_featured: Option<bool>,
    #[serde(default)]
    pub banner_image: Option<String>,
    #[serde(default)]
    pub publish_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCompetitionRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub organizer: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_bg: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub prize: Option<String>,
    #[serde(default)]
    pub team_size_min: Option<u8>,
    #[serde(default)]
    pub team_size_max: Option<u8>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_featured: Option<bool>,
    #[serde(default)]
    pub banner_image: Option<String>,
    #[serde(default)]
    pub publish_status: Option<String>,
    #[serde(default)]
    pub rejected_note: Option<String>,
}

/// API response — includes computed viewer-state fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionResponse {
    pub id: String,
    pub name: String,
    pub organizer: String,
    pub icon: String,
    pub icon_bg: String,
    pub status: String,
    pub prize: String,
    pub team_size_min: u8,
    pub team_size_max: u8,
    pub deadline: Option<String>,
    pub duration: String,
    pub tags: Vec<String>,
    pub description: String,
    pub is_featured: bool,
    pub banner_image: Option<String>,
    pub publish_status: String,
    pub publisher_id: Option<String>,
    pub rejected_note: Option<String>,
    pub winners: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Computed at response time
    pub registration_count: usize,
    pub interested_count: usize,

    /// Set at API response time based on the authenticated viewer.
    #[serde(default)]
    pub is_registered_by_viewer: Option<bool>,
    #[serde(default)]
    pub is_interested_by_viewer: Option<bool>,

    /// Include full registrations only for owners / admins.
    #[serde(default)]
    pub registrations: Option<Vec<Registration>>,
}

impl CompetitionResponse {
    /// Build a response from a Competition document.
    ///
    /// `viewer_id` is the currently authenticated user (if any).
    /// `include_registrations` controls whether the full list is exposed.
    pub fn from_competition(
        c: Competition,
        viewer_id: Option<&str>,
        include_registrations: bool,
    ) -> Self {
        let registration_count = c.registrations.len();
        let interested_count = c.interested_user_ids.len();
        let is_registered = viewer_id
            .map(|vid| c.registrations.iter().any(|r| r.user_id == vid));
        let is_interested = viewer_id
            .map(|vid| c.interested_user_ids.iter().any(|uid| uid == vid));

        Self {
            id: c.id,
            name: c.name,
            organizer: c.organizer,
            icon: c.icon,
            icon_bg: c.icon_bg,
            status: c.status,
            prize: c.prize,
            team_size_min: c.team_size_min,
            team_size_max: c.team_size_max,
            deadline: c.deadline,
            duration: c.duration,
            tags: c.tags,
            description: c.description,
            is_featured: c.is_featured,
            banner_image: c.banner_image,
            publish_status: c.publish_status,
            publisher_id: c.publisher_id,
            rejected_note: c.rejected_note,
            winners: c.winners,
            created_at: c.created_at,
            updated_at: c.updated_at,
            registration_count,
            interested_count,
            is_registered_by_viewer: is_registered,
            is_interested_by_viewer: is_interested,
            registrations: if include_registrations {
                Some(c.registrations)
            } else {
                None
            },
        }
    }
}
