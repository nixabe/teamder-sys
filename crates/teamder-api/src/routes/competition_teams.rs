use chrono::Utc;
use mongodb::bson;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::competition_team::{CompTeamMember, CompetitionTeam};
use teamder_core::models::join_request::JoinRequest;
use teamder_core::models::notification::Notification;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCompTeamBody {
    pub competition_id: String,
    pub competition_name: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub max_members: Option<u8>,
    #[serde(default)]
    pub looking_for: Option<Vec<String>>,
    #[serde(default)]
    pub open_roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompTeamBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub max_members: Option<u8>,
    #[serde(default)]
    pub looking_for: Option<Vec<String>>,
    #[serde(default)]
    pub open_roles: Option<Vec<String>>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyBody {
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::post("/competition-teams", data = "<body>")]
pub async fn create_team(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<CreateCompTeamBody>,
) -> Result<Json<CompetitionTeam>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    let user = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let lead_member = CompTeamMember {
        user_id: auth.user_id.clone(),
        name: user.name.clone(),
        initials: user.initials.clone(),
        role: Some("Lead".to_string()),
        joined_at: now,
    };

    let team = CompetitionTeam {
        id: id.clone(),
        competition_id: req.competition_id,
        competition_name: req.competition_name,
        name: req.name,
        description: req.description.unwrap_or_default(),
        lead_user_id: auth.user_id,
        members: vec![lead_member],
        max_members: req.max_members.unwrap_or(5),
        looking_for: req.looking_for.unwrap_or_default(),
        open_roles: req.open_roles.unwrap_or_default(),
        status: "recruiting".to_string(),
        created_at: now,
        updated_at: now,
    };

    state.db.competition_team_repo().create(&team).await?;

    Ok(Json(team))
}

#[rocket::get("/competition-teams/<id>")]
pub async fn get_team(
    state: &State<AppState>,
    id: &str,
) -> Result<Json<CompetitionTeam>, ApiError> {
    let team = state
        .db
        .competition_team_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;
    Ok(Json(team))
}

#[rocket::patch("/competition-teams/<id>", data = "<body>")]
pub async fn update_team(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<UpdateCompTeamBody>,
) -> Result<Json<CompetitionTeam>, ApiError> {
    let team = state
        .db
        .competition_team_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;

    if team.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the lead can edit this team".into()).into());
    }

    let req = body.into_inner();
    let mut update = bson::doc! {};

    if let Some(v) = &req.name { update.insert("name", v.as_str()); }
    if let Some(v) = &req.description { update.insert("description", v.as_str()); }
    if let Some(v) = req.max_members { update.insert("max_members", v as i32); }
    if let Some(v) = &req.looking_for {
        update.insert("looking_for", bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?);
    }
    if let Some(v) = &req.open_roles {
        update.insert("open_roles", bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?);
    }
    if let Some(v) = &req.status { update.insert("status", v.as_str()); }

    update.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));

    state.db.competition_team_repo().update(id, update).await?;

    let updated = state
        .db
        .competition_team_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;

    Ok(Json(updated))
}

#[rocket::post("/competition-teams/<id>/apply", data = "<body>")]
pub async fn apply_to_team(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<ApplyBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let team = state
        .db
        .competition_team_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;

    // Check not already a member
    if team.members.iter().any(|m| m.user_id == auth.user_id) {
        return Err(TeamderError::Conflict("Already a member".into()).into());
    }

    // Check for existing pending request
    if let Some(_) = state
        .db
        .join_request_repo()
        .find_pending_for_entity(&auth.user_id, id)
        .await?
    {
        return Err(TeamderError::Conflict("Already applied".into()).into());
    }

    let req = body.into_inner();
    let now = Utc::now();

    let jr = JoinRequest {
        id: Uuid::new_v4().to_string(),
        from_user_id: auth.user_id.clone(),
        entity_type: "competition_team".to_string(),
        entity_id: id.to_string(),
        entity_name: team.name.clone(),
        owner_id: team.lead_user_id.clone(),
        message: req.message,
        status: "pending".to_string(),
        motivation: None,
        role_wanted: None,
        hours_per_week: None,
        portfolio_url: None,
        relevant_experience: None,
        availability_start: None,
        can_meet_in_person: None,
        additional_links: vec![],
        comm_channels: vec![],
        timezone: None,
        agreed_to_coc: false,
        skill_confidence: vec![],
        created_at: now,
    };

    state.db.join_request_repo().create(&jr).await?;

    // Notify team lead
    let applicant = state.db.user_repo().find_by_id(&auth.user_id).await?;
    let notif = Notification {
        id: Uuid::new_v4().to_string(),
        user_id: team.lead_user_id,
        kind: "join_request".to_string(),
        title: format!(
            "{} wants to join {}",
            applicant.map(|u| u.name).unwrap_or_default(),
            team.name
        ),
        body: String::new(),
        link: Some(format!("/competition-teams/{}", id)),
        read: false,
        created_at: now,
    };
    let _ = state.db.notification_repo().create(&notif).await;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/competition-teams/<id>/accept/<user_id>")]
pub async fn accept_member(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    user_id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let team = state
        .db
        .competition_team_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;

    if team.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the lead can accept members".into()).into());
    }

    let new_user = state
        .db
        .user_repo()
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let member = CompTeamMember {
        user_id: user_id.to_string(),
        name: new_user.name.clone(),
        initials: new_user.initials.clone(),
        role: None,
        joined_at: Utc::now(),
    };

    state.db.competition_team_repo().add_member(id, &member).await?;

    // Auto-set "full" if members >= max_members
    let updated_team = state.db.competition_team_repo().find_by_id(id).await?;
    if let Some(t) = &updated_team {
        if t.members.len() >= t.max_members as usize {
            state.db.competition_team_repo().set_status(id, "full").await?;
        }
    }

    // Update the join request status if one exists
    if let Some(jr) = state
        .db
        .join_request_repo()
        .find_pending_for_entity(user_id, id)
        .await?
    {
        state
            .db
            .join_request_repo()
            .update_status(&jr.id, "accepted")
            .await?;
    }

    // Notify applicant
    let notif = Notification {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        kind: "join_accepted".to_string(),
        title: format!("You were accepted to {}", team.name),
        body: String::new(),
        link: Some(format!("/competition-teams/{}", id)),
        read: false,
        created_at: Utc::now(),
    };
    let _ = state.db.notification_repo().create(&notif).await;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::get("/competition-teams/<id>/applications")]
pub async fn list_applications(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<Vec<JoinRequest>>, ApiError> {
    let team = state
        .db
        .competition_team_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;

    if team.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the lead can view applications".into()).into());
    }

    // Get pending join requests for this team
    let requests = state.db.join_request_repo().incoming(&auth.user_id).await?;
    let filtered: Vec<JoinRequest> = requests
        .into_iter()
        .filter(|jr| jr.entity_id == id && jr.entity_type == "competition_team")
        .collect();

    Ok(Json(filtered))
}

#[rocket::post("/competition-teams/<id>/leave")]
pub async fn leave_team(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let team = state
        .db
        .competition_team_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;

    if team.lead_user_id == auth.user_id {
        return Err(TeamderError::Validation("Lead cannot leave the team".into()).into());
    }

    state
        .db
        .competition_team_repo()
        .remove_member(id, &auth.user_id)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::get("/competition-teams/competition/<comp_id>", rank = 1)]
pub async fn teams_by_competition(
    state: &State<AppState>,
    comp_id: &str,
) -> Result<Json<Vec<CompetitionTeam>>, ApiError> {
    let teams = state
        .db
        .competition_team_repo()
        .list_by_competition(comp_id)
        .await?;
    Ok(Json(teams))
}

#[rocket::get("/competition-teams/mine", rank = 1)]
pub async fn my_teams(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<CompetitionTeam>>, ApiError> {
    let teams = state
        .db
        .competition_team_repo()
        .find_by_lead(&auth.user_id)
        .await?;
    Ok(Json(teams))
}
