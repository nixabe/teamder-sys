use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::models::project::JoinMode;

fn default_join_mode() -> JoinMode { JoinMode::Direct }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyNote {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: String,
    pub initials: String,
    pub color: String,
    pub joined_at: DateTime<Utc>,
    pub last_checkin: Option<DateTime<Utc>>,
    pub streak: u32,
}

/// Core study group document in `study_groups` collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyGroup {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub goal: String,
    pub icon: String,
    pub icon_bg: String,
    pub subject: String,
    pub tags: Vec<String>,
    pub members: Vec<GroupMember>,
    pub max_members: u8,
    pub schedule: String,
    pub duration_weeks: u8,
    pub current_week: u8,
    pub is_open: bool,
    #[serde(default = "default_join_mode")]
    pub join_mode: JoinMode,
    #[serde(default)]
    pub banner_image: Option<String>,
    #[serde(default)]
    pub notes: Vec<StudyNote>,
    #[serde(default)]
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StudyGroup {
    pub fn new(
        name: impl Into<String>,
        goal: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            goal: goal.into(),
            icon: "Sg".into(),
            icon_bg: "linear-gradient(135deg, #4F6D7A, #6B8593)".into(),
            subject: "General".into(),
            tags: vec![],
            members: vec![],
            max_members: 6,
            schedule: "TBD".into(),
            duration_weeks: 8,
            current_week: 1,
            is_open: true,
            join_mode: JoinMode::Direct,
            banner_image: None,
            notes: vec![],
            description: None,
            created_by: created_by.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn progress_percent(&self) -> u8 {
        if self.duration_weeks == 0 {
            return 0;
        }
        ((self.current_week as f32 / self.duration_weeks as f32) * 100.0) as u8
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateStudyGroupRequest {
    pub name: String,
    pub goal: String,
    pub subject: String,
    pub tags: Vec<String>,
    pub max_members: Option<u8>,
    pub schedule: String,
    pub duration_weeks: u8,
    pub icon: Option<String>,
    pub icon_bg: Option<String>,
    pub join_mode: Option<JoinMode>,
    pub banner_image: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStudyNoteRequest {
    pub title: String,
    pub body: String,
}

/// A member entry enriched with the user's name (resolved at API layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberEnriched {
    pub user_id: String,
    pub name: String,
    pub initials: String,
    pub color: String,
    pub joined_at: DateTime<Utc>,
    pub streak: u32,
}

/// Full detail response used for joined/managed study groups.
#[derive(Debug, Serialize)]
pub struct StudyGroupDetail {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub icon: String,
    pub icon_bg: String,
    pub subject: String,
    pub tags: Vec<String>,
    pub members: Vec<GroupMemberEnriched>,
    pub max_members: u8,
    pub schedule: String,
    pub duration_weeks: u8,
    pub current_week: u8,
    pub progress_percent: u8,
    pub is_open: bool,
    pub join_mode: JoinMode,
    pub banner_image: Option<String>,
    pub notes: Vec<StudyNote>,
    pub description: Option<String>,
    pub created_by: String,
    pub creator_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StudyGroupResponse {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub icon: String,
    pub icon_bg: String,
    pub subject: String,
    pub tags: Vec<String>,
    pub member_count: usize,
    pub max_members: u8,
    pub schedule: String,
    pub duration_weeks: u8,
    pub current_week: u8,
    pub progress_percent: u8,
    pub is_open: bool,
    pub join_mode: JoinMode,
    pub banner_image: Option<String>,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl From<StudyGroup> for StudyGroupResponse {
    fn from(g: StudyGroup) -> Self {
        let progress = g.progress_percent();
        let count = g.members.len();
        Self {
            id: g.id,
            name: g.name,
            goal: g.goal,
            icon: g.icon,
            icon_bg: g.icon_bg,
            subject: g.subject,
            tags: g.tags,
            member_count: count,
            max_members: g.max_members,
            schedule: g.schedule,
            duration_weeks: g.duration_weeks,
            current_week: g.current_week,
            progress_percent: progress,
            is_open: g.is_open,
            join_mode: g.join_mode,
            banner_image: g.banner_image,
            description: g.description,
            created_by: g.created_by,
            created_at: g.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_group() -> StudyGroup {
        StudyGroup::new("Rust Study", "Learn Rust together", "user-1")
    }

    fn make_member(user_id: &str) -> GroupMember {
        GroupMember {
            user_id: user_id.into(),
            initials: "AB".into(),
            color: "#4F6D7A".into(),
            joined_at: Utc::now(),
            last_checkin: None,
            streak: 0,
        }
    }

    #[test]
    fn test_default_max_members() {
        let g = make_group();
        assert_eq!(g.max_members, 6);
    }

    #[test]
    fn test_default_is_open() {
        let g = make_group();
        assert!(g.is_open);
    }

    #[test]
    fn test_default_current_week_one() {
        let g = make_group();
        assert_eq!(g.current_week, 1);
    }

    #[test]
    fn test_default_members_empty() {
        let g = make_group();
        assert!(g.members.is_empty());
    }

    #[test]
    fn test_progress_percent_zero_current_week() {
        let mut g = make_group();
        g.current_week = 0;
        g.duration_weeks = 8;
        assert_eq!(g.progress_percent(), 0);
    }

    #[test]
    fn test_progress_percent_midpoint() {
        let mut g = make_group();
        g.current_week = 4;
        g.duration_weeks = 8;
        assert_eq!(g.progress_percent(), 50);
    }

    #[test]
    fn test_progress_percent_complete() {
        let mut g = make_group();
        g.current_week = 8;
        g.duration_weeks = 8;
        assert_eq!(g.progress_percent(), 100);
    }

    #[test]
    fn test_progress_percent_zero_duration_returns_zero() {
        let mut g = make_group();
        g.current_week = 1;
        g.duration_weeks = 0;
        assert_eq!(g.progress_percent(), 0);
    }

    #[test]
    fn test_response_member_count() {
        let mut g = make_group();
        g.members.push(make_member("u1"));
        g.members.push(make_member("u2"));
        let resp = StudyGroupResponse::from(g);
        assert_eq!(resp.member_count, 2);
    }

    #[test]
    fn test_response_progress_percent_propagated() {
        let mut g = make_group();
        g.current_week = 3;
        g.duration_weeks = 10;
        let resp = StudyGroupResponse::from(g);
        assert_eq!(resp.progress_percent, 30);
    }

    #[test]
    fn test_id_is_uuid_like() {
        let g = make_group();
        assert_eq!(g.id.len(), 36);
    }
}
