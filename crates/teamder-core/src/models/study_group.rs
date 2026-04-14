use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            created_at: g.created_at,
        }
    }
}
