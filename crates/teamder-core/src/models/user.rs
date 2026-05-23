use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Sub-structs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub level: u8, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItem {
    pub title: String,
    pub kind: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedReview {
    pub reviewer_id: String,
    pub reviewer_name: String,
    pub project_name: String,
    pub stars: u8,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

// ── Main document ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,

    pub email: String,
    pub password_hash: String,
    pub name: String,

    #[serde(default)]
    pub initials: String,

    #[serde(default)]
    pub role: String,

    #[serde(default)]
    pub department: String,

    #[serde(default = "default_university")]
    pub university: String,

    #[serde(default = "default_year")]
    pub year: String,

    #[serde(default)]
    pub location: Option<String>,

    #[serde(default)]
    pub bio: Vec<String>,

    #[serde(default)]
    pub skills: Vec<Skill>,

    #[serde(default)]
    pub skill_tags: Vec<String>,

    #[serde(default)]
    pub gradient: String,

    #[serde(default)]
    pub work_mode: Option<String>,

    #[serde(default)]
    pub availability: Option<String>,

    #[serde(default)]
    pub hours_per_week: Option<String>,

    #[serde(default = "default_languages")]
    pub languages: Vec<String>,

    #[serde(default)]
    pub portfolio: Vec<PortfolioItem>,

    #[serde(default)]
    pub reviews: Vec<CachedReview>,

    #[serde(default)]
    pub match_score: Option<u8>,

    #[serde(default)]
    pub rating: f32,

    #[serde(default)]
    pub projects_done: u32,

    #[serde(default)]
    pub collaborations: u32,

    #[serde(default)]
    pub avatar_url: Option<String>,

    #[serde(default)]
    pub resume_url: Option<String>,

    #[serde(default)]
    pub reset_token: Option<String>,

    #[serde(default)]
    pub reset_token_expires_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub is_admin: bool,

    #[serde(default)]
    pub is_publisher: bool,

    #[serde(default = "default_true")]
    pub is_public: bool,

    #[serde(default)]
    pub onboarded: bool,

    #[serde(default)]
    pub headline: Option<String>,

    #[serde(default = "default_true")]
    pub notify_email: bool,

    #[serde(default = "default_true")]
    pub notify_in_app: bool,

    #[serde(default)]
    pub social_links: Vec<SocialLink>,

    #[serde(default)]
    pub interests: Vec<String>,

    #[serde(default)]
    pub timezone: Option<String>,

    #[serde(default)]
    pub goals: Option<String>,

    #[serde(default)]
    pub free_days: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Default helpers ──────────────────────────────────────────────────────────

fn default_university() -> String {
    "Fu Jen Catholic University".to_string()
}

fn default_year() -> String {
    "N/A".to_string()
}

fn default_languages() -> Vec<String> {
    vec!["Chinese".to_string(), "English".to_string()]
}

fn default_true() -> bool {
    true
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Public-facing user response — strips sensitive fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub work_mode: Option<String>,
    pub availability: Option<String>,
    pub hours_per_week: Option<String>,
    pub languages: Vec<String>,
    pub portfolio: Vec<PortfolioItem>,
    pub reviews: Vec<CachedReview>,
    pub match_score: Option<u8>,
    pub rating: f32,
    pub projects_done: u32,
    pub collaborations: u32,
    pub avatar_url: Option<String>,
    pub resume_url: Option<String>,
    pub is_admin: bool,
    pub is_publisher: bool,
    pub is_public: bool,
    pub onboarded: bool,
    pub headline: Option<String>,
    pub notify_email: bool,
    pub notify_in_app: bool,
    pub social_links: Vec<SocialLink>,
    pub interests: Vec<String>,
    pub timezone: Option<String>,
    pub goals: Option<String>,
    pub free_days: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
            avatar_url: u.avatar_url,
            resume_url: u.resume_url,
            is_admin: u.is_admin,
            is_publisher: u.is_publisher,
            is_public: u.is_public,
            onboarded: u.onboarded,
            headline: u.headline,
            notify_email: u.notify_email,
            notify_in_app: u.notify_in_app,
            social_links: u.social_links,
            interests: u.interests,
            timezone: u.timezone,
            goals: u.goals,
            free_days: u.free_days,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// Registration / create-user payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub university: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub bio: Option<Vec<String>>,
    #[serde(default)]
    pub skills: Option<Vec<Skill>>,
    #[serde(default)]
    pub skill_tags: Option<Vec<String>>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub interests: Option<Vec<String>>,
}

/// Partial-update payload — every field optional.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUserRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub initials: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub university: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub bio: Option<Vec<String>>,
    #[serde(default)]
    pub skills: Option<Vec<Skill>>,
    #[serde(default)]
    pub skill_tags: Option<Vec<String>>,
    #[serde(default)]
    pub gradient: Option<String>,
    #[serde(default)]
    pub work_mode: Option<String>,
    #[serde(default)]
    pub availability: Option<String>,
    #[serde(default)]
    pub hours_per_week: Option<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub portfolio: Option<Vec<PortfolioItem>>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub resume_url: Option<String>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub onboarded: Option<bool>,
    #[serde(default)]
    pub headline: Option<String>,
    #[serde(default)]
    pub notify_email: Option<bool>,
    #[serde(default)]
    pub notify_in_app: Option<bool>,
    #[serde(default)]
    pub social_links: Option<Vec<SocialLink>>,
    #[serde(default)]
    pub interests: Option<Vec<String>>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub goals: Option<String>,
    #[serde(default)]
    pub free_days: Option<Vec<String>>,
    #[serde(default)]
    pub password: Option<String>,
}
