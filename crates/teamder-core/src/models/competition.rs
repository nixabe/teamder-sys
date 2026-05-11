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
    #[serde(default)]
    pub banner_image: Option<String>,
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
    pub registration_count: usize,
    pub interested_count: usize,
    pub winners: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Competition> for CompetitionResponse {
    fn from(c: Competition) -> Self {
        let count = c.registrations.len();
        let interested = c.interested_user_ids.len();
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
            registration_count: count,
            interested_count: interested,
            winners: c.winners,
            created_at: c.created_at,
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
        });
        c.registrations.push(Registration {
            user_id: "u2".into(),
            team_name: Some("Team A".into()),
            registered_at: Utc::now(),
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
}
