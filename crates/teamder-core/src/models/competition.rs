use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CompetitionStatus {
    Open,
    ClosingSoon,
    Upcoming,
    Past,
}

/// Registration record for a competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub user_id: String,
    pub team_name: Option<String>,
    pub registered_at: DateTime<Utc>,
}

/// Core competition document in `competitions` collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub organizer: String,
    pub icon: String,
    pub icon_bg: String,
    pub status: CompetitionStatus,
    pub prize: String,
    pub team_size_min: u8,
    pub team_size_max: u8,
    pub deadline: Option<String>,
    pub duration: String,
    pub tags: Vec<String>,
    pub description: String,
    pub is_featured: bool,
    pub registrations: Vec<Registration>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Competition {
    pub fn new(
        name: impl Into<String>,
        organizer: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            organizer: organizer.into(),
            icon: "Cp".into(),
            icon_bg: "linear-gradient(135deg, #DD6E42, #B85530)".into(),
            status: CompetitionStatus::Upcoming,
            prize: "TBD".into(),
            team_size_min: 2,
            team_size_max: 5,
            deadline: None,
            duration: "TBD".into(),
            tags: vec![],
            description: description.into(),
            is_featured: false,
            registrations: vec![],
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCompetitionRequest {
    pub name: String,
    pub organizer: String,
    pub description: String,
    pub prize: String,
    pub team_size_min: u8,
    pub team_size_max: u8,
    pub deadline: Option<String>,
    pub duration: String,
    pub tags: Vec<String>,
    pub is_featured: Option<bool>,
    pub icon: Option<String>,
    pub icon_bg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterCompetitionRequest {
    pub team_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompetitionResponse {
    pub id: String,
    pub name: String,
    pub organizer: String,
    pub icon: String,
    pub icon_bg: String,
    pub status: CompetitionStatus,
    pub prize: String,
    pub team_size_min: u8,
    pub team_size_max: u8,
    pub deadline: Option<String>,
    pub duration: String,
    pub tags: Vec<String>,
    pub description: String,
    pub is_featured: bool,
    pub registration_count: usize,
    pub created_at: DateTime<Utc>,
}

impl From<Competition> for CompetitionResponse {
    fn from(c: Competition) -> Self {
        let count = c.registrations.len();
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
            registration_count: count,
            created_at: c.created_at,
        }
    }
}
