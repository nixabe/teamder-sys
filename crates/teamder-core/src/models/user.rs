use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Collaboration work mode preference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkMode {
    Remote,
    Hybrid,
    InPerson,
}

/// Availability status visible on the platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    OpenForCollab,
    Busy,
    Unavailable,
}

/// A skill tag with self-reported proficiency level (0–100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    /// 0–100 proficiency score
    pub level: u8,
}

/// Portfolio piece shown on a user's profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItem {
    pub title: String,
    pub kind: String,
    pub description: Option<String>,
    pub url: Option<String>,
}

/// Review left by a collaborator after a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub reviewer_id: String,
    pub reviewer_name: String,
    pub project_name: String,
    /// 1–5
    pub stars: u8,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

/// Core user document stored in MongoDB `users` collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    #[serde(default, skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub initials: String,
    pub role: String,
    pub department: String,
    pub university: String,
    pub year: String,
    pub location: Option<String>,
    pub bio: Vec<String>,
    pub skills: Vec<Skill>,
    pub skill_tags: Vec<String>,
    pub gradient: String,
    pub work_mode: WorkMode,
    pub availability: AvailabilityStatus,
    pub hours_per_week: String,
    pub languages: Vec<String>,
    pub portfolio: Vec<PortfolioItem>,
    pub reviews: Vec<Review>,
    pub match_score: u8,
    pub rating: f32,
    pub projects_done: u32,
    pub collaborations: u32,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: impl Into<String>,
        password_hash: impl Into<String>,
        name: impl Into<String>,
        role: impl Into<String>,
        department: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let initials: String = name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.into(),
            password_hash: password_hash.into(),
            name,
            initials,
            role: role.into(),
            department: department.into(),
            university: "Fu Jen Catholic University".into(),
            year: "N/A".into(),
            location: None,
            bio: vec![],
            skills: vec![],
            skill_tags: vec![],
            gradient: "linear-gradient(135deg, #DD6E42, #E89070)".into(),
            work_mode: WorkMode::Hybrid,
            availability: AvailabilityStatus::OpenForCollab,
            hours_per_week: "10-15 hrs/week".into(),
            languages: vec!["Chinese".into(), "English".into()],
            portfolio: vec![],
            reviews: vec![],
            match_score: 0,
            rating: 0.0,
            projects_done: 0,
            collaborations: 0,
            is_admin: false,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Payload for creating a new user account.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: String,
    pub department: String,
}

/// Payload for updating profile fields.
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub year: Option<String>,
    pub location: Option<String>,
    pub bio: Option<Vec<String>>,
    pub skills: Option<Vec<Skill>>,
    pub skill_tags: Option<Vec<String>>,
    pub work_mode: Option<WorkMode>,
    pub hours_per_week: Option<String>,
    pub languages: Option<Vec<String>>,
}

/// Response shape returned to API clients (no password hash).
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub initials: String,
    pub role: String,
    pub department: String,
    pub university: String,
    pub year: String,
    pub location: Option<String>,
    pub bio: Vec<String>,
    pub skills: Vec<Skill>,
    pub skill_tags: Vec<String>,
    pub gradient: String,
    pub work_mode: WorkMode,
    pub availability: AvailabilityStatus,
    pub hours_per_week: String,
    pub languages: Vec<String>,
    pub portfolio: Vec<PortfolioItem>,
    pub reviews: Vec<Review>,
    pub match_score: u8,
    pub rating: f32,
    pub projects_done: u32,
    pub collaborations: u32,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            name: u.name,
            initials: u.initials,
            role: u.role,
            department: u.department,
            university: u.university,
            year: u.year,
            location: u.location,
            bio: u.bio,
            skills: u.skills,
            skill_tags: u.skill_tags,
            gradient: u.gradient,
            work_mode: u.work_mode,
            availability: u.availability,
            hours_per_week: u.hours_per_week,
            languages: u.languages,
            portfolio: u.portfolio,
            reviews: u.reviews,
            match_score: u.match_score,
            rating: u.rating,
            projects_done: u.projects_done,
            collaborations: u.collaborations,
            created_at: u.created_at,
        }
    }
}
