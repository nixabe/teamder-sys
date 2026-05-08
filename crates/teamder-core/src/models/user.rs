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

/// External social / professional link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    /// Display label e.g. "GitHub", "LinkedIn", "Personal site".
    pub label: String,
    pub url: String,
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
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub resume_url: Option<String>,
    /// Single-use password reset token (hex). Cleared after a successful reset.
    #[serde(default)]
    pub reset_token: Option<String>,
    /// Expiry for the reset token (RFC3339 string for portability).
    #[serde(default)]
    pub reset_token_expires_at: Option<DateTime<Utc>>,
    /// Has the user completed the onboarding wizard?
    #[serde(default)]
    pub onboarded: bool,
    /// Short one-liner shown above the bio (e.g. "Designer + frontend dev").
    #[serde(default)]
    pub headline: Option<String>,
    /// Email notifications opt-in (kept here so the prefs travel with the user).
    #[serde(default = "default_true")]
    pub notify_email: bool,
    /// In-app notifications opt-in.
    #[serde(default = "default_true")]
    pub notify_in_app: bool,
    /// Public profile flag. When false, the profile is hidden from search.
    #[serde(default = "default_true")]
    pub is_public: bool,
    /// Social / professional links — displayed on the public profile.
    #[serde(default)]
    pub social_links: Vec<SocialLink>,
    /// Free-form interest tags ("side projects", "research", "hackathons", …).
    #[serde(default)]
    pub interests: Vec<String>,
    /// Time-zone string (IANA) used for scheduling hints. Optional.
    #[serde(default)]
    pub timezone: Option<String>,
    /// Long-form personal goals — what the user is hoping to find / build.
    #[serde(default)]
    pub goals: Option<String>,
    /// Days of the week the user is generally available (e.g. ["Mon","Wed","Fri"]).
    #[serde(default)]
    pub free_days: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_true() -> bool { true }

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
            avatar_url: None,
            resume_url: None,
            reset_token: None,
            reset_token_expires_at: None,
            onboarded: false,
            headline: None,
            notify_email: true,
            notify_in_app: true,
            is_public: true,
            social_links: vec![],
            interests: vec![],
            timezone: None,
            goals: None,
            free_days: vec![],
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
    /// Optional school / institution. Falls back to "Fu Jen Catholic University"
    /// (the platform's home campus) when missing.
    #[serde(default)]
    pub university: Option<String>,
    /// Optional year-of-study label, e.g. "Year 3" / "Graduate".
    #[serde(default)]
    pub year: Option<String>,
    /// Optional one-line bio shown next to the name.
    #[serde(default)]
    pub headline: Option<String>,
    /// Optional location string ("Taipei, Taiwan").
    #[serde(default)]
    pub location: Option<String>,
    /// Optional preferred work mode set during signup.
    #[serde(default)]
    pub work_mode: Option<WorkMode>,
    /// Optional weekly hours descriptor.
    #[serde(default)]
    pub hours_per_week: Option<String>,
    /// Optional spoken languages list.
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    /// Optional social / professional links.
    #[serde(default)]
    pub social_links: Option<Vec<SocialLink>>,
    /// Optional interest tags.
    #[serde(default)]
    pub interests: Option<Vec<String>>,
    /// Optional time-zone (IANA string).
    #[serde(default)]
    pub timezone: Option<String>,
    /// Optional long-form goals statement.
    #[serde(default)]
    pub goals: Option<String>,
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
    pub availability: Option<AvailabilityStatus>,
    pub hours_per_week: Option<String>,
    pub languages: Option<Vec<String>>,
    pub portfolio: Option<Vec<PortfolioItem>>,
    pub avatar_url: Option<Option<String>>,
    pub resume_url: Option<Option<String>>,
    pub onboarded: Option<bool>,
    pub headline: Option<Option<String>>,
    pub notify_email: Option<bool>,
    pub notify_in_app: Option<bool>,
    pub is_public: Option<bool>,
    pub social_links: Option<Vec<SocialLink>>,
    pub interests: Option<Vec<String>>,
    pub timezone: Option<Option<String>>,
    pub goals: Option<Option<String>>,
    /// Days of the week the user is generally available.
    #[serde(default)]
    pub free_days: Option<Vec<String>>,
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
    pub avatar_url: Option<String>,
    pub resume_url: Option<String>,
    pub is_admin: bool,
    pub onboarded: bool,
    pub headline: Option<String>,
    pub notify_email: bool,
    pub notify_in_app: bool,
    pub is_public: bool,
    pub social_links: Vec<SocialLink>,
    pub interests: Vec<String>,
    pub timezone: Option<String>,
    pub goals: Option<String>,
    pub free_days: Vec<String>,
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
            avatar_url: u.avatar_url,
            resume_url: u.resume_url,
            is_admin: u.is_admin,
            onboarded: u.onboarded,
            headline: u.headline,
            notify_email: u.notify_email,
            notify_in_app: u.notify_in_app,
            is_public: u.is_public,
            social_links: u.social_links,
            interests: u.interests,
            timezone: u.timezone,
            goals: u.goals,
            free_days: u.free_days,
            created_at: u.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initials_two_words() {
        let u = User::new("a@b.com", "hash", "Alice Wang", "Dev", "CS");
        assert_eq!(u.initials, "AW");
    }

    #[test]
    fn test_initials_single_word() {
        let u = User::new("a@b.com", "hash", "Alice", "Dev", "CS");
        assert_eq!(u.initials, "A");
    }

    #[test]
    fn test_initials_many_words_capped_at_two() {
        let u = User::new("a@b.com", "hash", "Alice Bob Chen", "Dev", "CS");
        assert_eq!(u.initials, "AB");
    }

    #[test]
    fn test_initials_lowercase_input_is_uppercased() {
        let u = User::new("a@b.com", "hash", "alice wang", "Dev", "CS");
        assert_eq!(u.initials, "AW");
    }

    #[test]
    fn test_default_is_not_admin() {
        let u = User::new("a@b.com", "hash", "Alice Wang", "Dev", "CS");
        assert!(!u.is_admin);
    }

    #[test]
    fn test_default_availability_open_for_collab() {
        let u = User::new("a@b.com", "hash", "Alice Wang", "Dev", "CS");
        assert_eq!(u.availability, AvailabilityStatus::OpenForCollab);
    }

    #[test]
    fn test_default_work_mode_hybrid() {
        let u = User::new("a@b.com", "hash", "Alice Wang", "Dev", "CS");
        assert_eq!(u.work_mode, WorkMode::Hybrid);
    }

    #[test]
    fn test_default_match_score_zero() {
        let u = User::new("a@b.com", "hash", "Alice Wang", "Dev", "CS");
        assert_eq!(u.match_score, 0);
    }

    #[test]
    fn test_id_is_uuid_like() {
        let u = User::new("a@b.com", "hash", "Alice Wang", "Dev", "CS");
        // UUID v4 has 36 chars with hyphens
        assert_eq!(u.id.len(), 36);
        assert!(u.id.contains('-'));
    }

    #[test]
    fn test_response_strips_password_hash() {
        let u = User::new("a@b.com", "secret_hash", "Alice Wang", "Dev", "CS");
        let resp = UserResponse::from(u.clone());
        // The response type doesn't even have a password_hash field
        assert_eq!(resp.id, u.id);
        assert_eq!(resp.email, u.email);
        assert_eq!(resp.name, u.name);
    }

    #[test]
    fn test_two_users_have_different_ids() {
        let u1 = User::new("a@b.com", "h", "Alice Wang", "Dev", "CS");
        let u2 = User::new("b@c.com", "h", "Bob Lin", "Dev", "CS");
        assert_ne!(u1.id, u2.id);
    }
}
