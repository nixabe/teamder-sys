use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Recruiting,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JoinMode {
    Direct,
    Approval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CollabMode {
    Remote,
    Hybrid,
    InPerson,
}

/// An open role slot within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRole {
    pub name: String,
    pub count_needed: u8,
    #[serde(default)]
    pub filled: u8,
}

/// A slim reference to a team member stored in the project doc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: String,
    pub initials: String,
    pub color: String,
    pub joined_at: DateTime<Utc>,
}

/// TeamMember enriched with the user's resolved name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberEnriched {
    pub user_id: String,
    pub name: String,
    pub initials: String,
    pub color: String,
    pub joined_at: DateTime<Utc>,
}

/// Core project document in `projects` collection. lead_name resolved at API layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub lead_user_id: String,
    pub icon: String,
    pub icon_bg: String,
    pub status: ProjectStatus,
    pub description: String,
    pub goals: Option<String>,
    pub roles: Vec<ProjectRole>,
    pub skills: Vec<String>,
    pub team: Vec<TeamMember>,
    pub deadline: Option<String>,
    pub collab: CollabMode,
    pub duration: Option<String>,
    pub category: Option<String>,
    pub is_public: bool,
    #[serde(default = "default_join_mode")]
    pub join_mode: JoinMode,
    /// Paid/admin-promoted flag — surfaces this project at the top of listings.
    #[serde(default)]
    pub is_promoted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_join_mode() -> JoinMode { JoinMode::Direct }

impl Project {
    pub fn new(
        name: impl Into<String>,
        lead_user_id: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            lead_user_id: lead_user_id.into(),
            icon: "Pr".into(),
            icon_bg: "linear-gradient(135deg, #4F6D7A, #2C3E45)".into(),
            status: ProjectStatus::Recruiting,
            description: description.into(),
            goals: None,
            roles: vec![],
            skills: vec![],
            team: vec![],
            deadline: None,
            collab: CollabMode::Hybrid,
            duration: None,
            category: None,
            is_public: true,
            join_mode: JoinMode::Direct,
            is_promoted: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
    pub goals: Option<String>,
    pub roles: Option<Vec<ProjectRole>>,
    pub skills: Vec<String>,
    pub deadline: Option<String>,
    pub collab: CollabMode,
    pub duration: Option<String>,
    pub category: Option<String>,
    pub is_public: Option<bool>,
    pub icon: Option<String>,
    pub icon_bg: Option<String>,
    pub join_mode: Option<JoinMode>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub goals: Option<String>,
    pub status: Option<ProjectStatus>,
    pub roles: Option<Vec<ProjectRole>>,
    pub skills: Option<Vec<String>>,
    pub deadline: Option<String>,
    pub collab: Option<CollabMode>,
    pub duration: Option<String>,
    pub is_public: Option<bool>,
    pub join_mode: Option<JoinMode>,
}

/// Full project detail with enriched member names, used for joined/managed views.
#[derive(Debug, Serialize)]
pub struct ProjectDetail {
    pub id: String,
    pub name: String,
    pub lead_user_id: String,
    pub lead_name: String,
    pub icon: String,
    pub icon_bg: String,
    pub status: ProjectStatus,
    pub description: String,
    pub goals: Option<String>,
    pub roles: Vec<ProjectRole>,
    pub skills: Vec<String>,
    pub team: Vec<TeamMemberEnriched>,
    pub deadline: Option<String>,
    pub collab: CollabMode,
    pub duration: Option<String>,
    pub category: Option<String>,
    pub is_public: bool,
    pub join_mode: JoinMode,
    pub is_promoted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub lead_user_id: String,
    pub lead_name: String,
    pub icon: String,
    pub icon_bg: String,
    pub status: ProjectStatus,
    pub description: String,
    pub goals: Option<String>,
    pub roles: Vec<ProjectRole>,
    pub skills: Vec<String>,
    pub team: Vec<TeamMember>,
    pub deadline: Option<String>,
    pub collab: CollabMode,
    pub duration: Option<String>,
    pub category: Option<String>,
    pub is_public: bool,
    pub join_mode: JoinMode,
    pub is_promoted: bool,
    pub created_at: DateTime<Utc>,
}

impl ProjectResponse {
    pub fn from_project(p: Project, lead_name: String) -> Self {
        Self {
            id: p.id,
            name: p.name,
            lead_user_id: p.lead_user_id,
            lead_name,
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
            created_at: p.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_project() -> Project {
        Project::new("Test Project", "user-1", "A test project")
    }

    #[test]
    fn test_default_status_recruiting() {
        let p = make_project();
        assert_eq!(p.status, ProjectStatus::Recruiting);
    }

    #[test]
    fn test_default_collab_hybrid() {
        let p = make_project();
        assert_eq!(p.collab, CollabMode::Hybrid);
    }

    #[test]
    fn test_default_is_public_true() {
        let p = make_project();
        assert!(p.is_public);
    }

    #[test]
    fn test_default_team_empty() {
        let p = make_project();
        assert!(p.team.is_empty());
    }

    #[test]
    fn test_default_goals_none() {
        let p = make_project();
        assert!(p.goals.is_none());
    }

    #[test]
    fn test_id_is_uuid_like() {
        let p = make_project();
        assert_eq!(p.id.len(), 36);
    }

    #[test]
    fn test_name_stored() {
        let p = make_project();
        assert_eq!(p.name, "Test Project");
    }

    #[test]
    fn test_response_from_project() {
        let p = make_project();
        let resp = ProjectResponse::from_project(p.clone(), "Alice Wang".into());
        assert_eq!(resp.id, p.id);
        assert_eq!(resp.name, p.name);
        assert_eq!(resp.lead_user_id, p.lead_user_id);
        assert_eq!(resp.lead_name, "Alice Wang");
        assert_eq!(resp.status, ProjectStatus::Recruiting);
        assert!(resp.is_public);
    }

    #[test]
    fn test_two_projects_have_different_ids() {
        let p1 = make_project();
        let p2 = make_project();
        assert_ne!(p1.id, p2.id);
    }
}
