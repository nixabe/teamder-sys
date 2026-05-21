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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublishStatus {
    Draft,
    PendingReview,
    Published,
    Rejected,
}

fn default_publish_status() -> PublishStatus {
    PublishStatus::Published
}

/// Registration record for a competition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub user_id: String,
    pub team_name: Option<String>,
    pub registered_at: DateTime<Utc>,
    #[serde(default)]
    pub motivation: Option<String>,
    #[serde(default)]
    pub skills: Option<String>,
    #[serde(default)]
    pub contact_email: Option<String>,
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
    #[serde(default)]
    pub banner_image: Option<String>,
    #[serde(default = "default_publish_status")]
    pub publish_status: PublishStatus,
    #[serde(default)]
    pub publisher_id: Option<String>,
    #[serde(default)]
    pub rejected_note: Option<String>,
    pub registrations: Vec<Registration>,
    /// User IDs who clicked "I'm interested" — used for the interest counter
    /// and to recommend the competition to similar users.
    #[serde(default)]
    pub interested_user_ids: Vec<String>,
    /// Optional winner roster, set after the competition concludes.
    #[serde(default)]
    pub winners: Vec<String>,
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
            banner_image: None,
            publish_status: PublishStatus::Draft,
            publisher_id: None,
            rejected_note: None,
            registrations: vec![],
            interested_user_ids: vec![],
            winners: vec![],
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
    pub banner_image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterCompetitionRequest {
    pub team_name: Option<String>,
    #[serde(default)]
    pub motivation: Option<String>,
    #[serde(default)]
    pub skills: Option<String>,
    #[serde(default)]
    pub contact_email: Option<String>,
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
    pub banner_image: Option<String>,
    pub publish_status: PublishStatus,
    pub publisher_id: Option<String>,
    pub rejected_note: Option<String>,
    pub registration_count: usize,
    pub interested_count: usize,
    pub winners: Vec<String>,
    pub created_at: DateTime<Utc>,
    /// True when the request was made by an authenticated user who has
    /// already registered for this competition. Lets the frontend show
    /// "✓ Registered" instead of "Register" without an extra round-trip.
    #[serde(default)]
    pub is_registered_by_viewer: bool,
    /// True when the authenticated viewer has marked themselves as
    /// interested. Used to toggle the ☆/★ Interested label.
    #[serde(default)]
    pub is_interested_by_viewer: bool,
}

#[derive(Debug, Deserialize)]
pub struct RejectCompetitionRequest {
    pub note: Option<String>,
}

impl From<Competition> for CompetitionResponse {
    fn from(c: Competition) -> Self {
        CompetitionResponse::from_competition(c, None)
    }
}

impl CompetitionResponse {
    /// Build a response tagged with whether `viewer_id` already registered
    /// or marked interest. Pass `None` for unauthenticated requests.
    pub fn from_competition(c: Competition, viewer_id: Option<&str>) -> Self {
        let count = c.registrations.len();
        let interested = c.interested_user_ids.len();
        let registered_by_viewer = viewer_id
            .map(|vid| c.registrations.iter().any(|r| r.user_id == vid))
            .unwrap_or(false);
        let interested_by_viewer = viewer_id
            .map(|vid| c.interested_user_ids.iter().any(|u| u == vid))
            .unwrap_or(false);
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
            registration_count: count,
            interested_count: interested,
            winners: c.winners,
            created_at: c.created_at,
            is_registered_by_viewer: registered_by_viewer,
            is_interested_by_viewer: interested_by_viewer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_competition() -> Competition {
        Competition::new("Test Hackathon", "FJCU", "A great hackathon for students")
    }

    #[test]
    fn test_default_status_upcoming() {
        let c = make_competition();
        assert_eq!(c.status, CompetitionStatus::Upcoming);
    }

    #[test]
    fn test_default_not_featured() {
        let c = make_competition();
        assert!(!c.is_featured);
    }

    #[test]
    fn test_default_registrations_empty() {
        let c = make_competition();
        assert!(c.registrations.is_empty());
    }

    #[test]
    fn test_default_team_size() {
        let c = make_competition();
        assert_eq!(c.team_size_min, 2);
        assert_eq!(c.team_size_max, 5);
    }

    #[test]
    fn test_id_is_uuid_like() {
        let c = make_competition();
        assert_eq!(c.id.len(), 36);
    }

    #[test]
    fn test_response_registration_count_zero() {
        let c = make_competition();
        let resp = CompetitionResponse::from(c);
        assert_eq!(resp.registration_count, 0);
    }

    #[test]
    fn test_response_registration_count_nonzero() {
        let mut c = make_competition();
        c.registrations.push(Registration {
            user_id: "u1".into(),
            team_name: None,
            registered_at: Utc::now(),
            motivation: None,
            skills: None,
            contact_email: None,
        });
        c.registrations.push(Registration {
            user_id: "u2".into(),
            team_name: Some("Team A".into()),
            registered_at: Utc::now(),
            motivation: None,
            skills: None,
            contact_email: None,
        });
        let resp = CompetitionResponse::from(c);
        assert_eq!(resp.registration_count, 2);
    }

    #[test]
    fn test_name_and_organizer_stored() {
        let c = make_competition();
        assert_eq!(c.name, "Test Hackathon");
        assert_eq!(c.organizer, "FJCU");
    }

    #[test]
    fn test_new_competition_default_publish_status_draft() {
        let c = make_competition();
        assert_eq!(c.publish_status, PublishStatus::Draft);
    }

    #[test]
    fn test_new_competition_no_publisher_id() {
        let c = make_competition();
        assert!(c.publisher_id.is_none());
        assert!(c.rejected_note.is_none());
    }

    #[test]
    fn test_default_publish_status_serde_backward_compat() {
        // Documents without publish_status field should default to Published
        let json = r#"{"_id":"x","name":"N","organizer":"O","icon":"I","icon_bg":"bg","status":"upcoming","prize":"P","team_size_min":2,"team_size_max":5,"duration":"1m","tags":[],"description":"D","is_featured":false,"registrations":[],"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#;
        let c: Competition = serde_json::from_str(json).unwrap();
        assert_eq!(c.publish_status, PublishStatus::Published);
    }

    #[test]
    fn test_publish_status_serde_round_trip() {
        let s = serde_json::to_string(&PublishStatus::PendingReview).unwrap();
        assert_eq!(s, "\"pending_review\"");
        let back: PublishStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(back, PublishStatus::PendingReview);
    }

    #[test]
    fn test_response_maps_publish_status_and_publisher_id() {
        let mut c = make_competition();
        c.publish_status = PublishStatus::Rejected;
        c.publisher_id = Some("pub-1".into());
        c.rejected_note = Some("Needs more detail".into());
        let resp = CompetitionResponse::from(c);
        assert_eq!(resp.publish_status, PublishStatus::Rejected);
        assert_eq!(resp.publisher_id, Some("pub-1".into()));
        assert_eq!(resp.rejected_note, Some("Needs more detail".into()));
    }
}
