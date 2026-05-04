use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A team formed around a single competition. Members coordinate inside the
/// team; once full, the team can be officially registered for the competition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CompetitionTeamStatus {
    /// Looking for additional members.
    Recruiting,
    /// All slots filled, no more applications accepted.
    Full,
    /// Officially submitted to the competition.
    Registered,
    /// Team disbanded or closed.
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionTeamMember {
    pub user_id: String,
    pub name: String,
    pub initials: String,
    /// Their role/responsibility on the team (e.g. "Frontend dev", "PM").
    #[serde(default)]
    pub role: Option<String>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionTeam {
    #[serde(rename = "_id")]
    pub id: String,
    pub competition_id: String,
    pub competition_name: String,
    pub name: String,
    pub description: String,
    pub lead_user_id: String,
    pub members: Vec<CompetitionTeamMember>,
    pub max_members: u8,
    /// Skill tags the team is looking for.
    pub looking_for: Vec<String>,
    /// Roles still open (e.g. ["Designer", "Backend"]).
    #[serde(default)]
    pub open_roles: Vec<String>,
    pub status: CompetitionTeamStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CompetitionTeam {
    pub fn new(
        competition_id: impl Into<String>,
        competition_name: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        lead_user_id: impl Into<String>,
        lead: CompetitionTeamMember,
        max_members: u8,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            competition_id: competition_id.into(),
            competition_name: competition_name.into(),
            name: name.into(),
            description: description.into(),
            lead_user_id: lead_user_id.into(),
            members: vec![lead],
            max_members,
            looking_for: vec![],
            open_roles: vec![],
            status: CompetitionTeamStatus::Recruiting,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCompetitionTeamRequest {
    pub competition_id: String,
    pub name: String,
    pub description: String,
    pub max_members: u8,
    #[serde(default)]
    pub looking_for: Vec<String>,
    #[serde(default)]
    pub open_roles: Vec<String>,
    /// Lead's role on the team.
    #[serde(default)]
    pub lead_role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompetitionTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_members: Option<u8>,
    pub looking_for: Option<Vec<String>>,
    pub open_roles: Option<Vec<String>>,
    pub status: Option<CompetitionTeamStatus>,
}

#[derive(Debug, Serialize)]
pub struct CompetitionTeamResponse {
    pub id: String,
    pub competition_id: String,
    pub competition_name: String,
    pub name: String,
    pub description: String,
    pub lead_user_id: String,
    pub lead_name: String,
    pub members: Vec<CompetitionTeamMember>,
    pub max_members: u8,
    pub member_count: usize,
    pub looking_for: Vec<String>,
    pub open_roles: Vec<String>,
    pub status: CompetitionTeamStatus,
    pub created_at: DateTime<Utc>,
}

impl CompetitionTeamResponse {
    pub fn from_team(t: CompetitionTeam, lead_name: String) -> Self {
        let count = t.members.len();
        Self {
            id: t.id,
            competition_id: t.competition_id,
            competition_name: t.competition_name,
            name: t.name,
            description: t.description,
            lead_user_id: t.lead_user_id,
            lead_name,
            members: t.members,
            max_members: t.max_members,
            member_count: count,
            looking_for: t.looking_for,
            open_roles: t.open_roles,
            status: t.status,
            created_at: t.created_at,
        }
    }
}
