use chrono::Utc;
use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::{
        competition_team::{
            CompetitionTeam, CompetitionTeamMember, CompetitionTeamResponse, CompetitionTeamStatus,
            CreateCompetitionTeamRequest, UpdateCompetitionTeamRequest,
        },
        join_request::{JoinRequest, JoinRequestResponse, JoinRequestStatus},
        notification::{Notification, NotificationKind},
    },
    skills::filter_valid_skills,
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

fn to_response(team: CompetitionTeam, lead_name: String) -> CompetitionTeamResponse {
    CompetitionTeamResponse::from_team(team, lead_name)
}

/// GET /api/v1/competition-teams?competition_id=…
#[get("/?<competition_id>")]
async fn list_teams(
    competition_id: Option<String>,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let teams = if let Some(cid) = competition_id {
        state.competition_teams.list_for_competition(&cid).await?
    } else {
        vec![]
    };
    let mut data = Vec::with_capacity(teams.len());
    for t in teams {
        let lead_name = state.users.find_by_id(&t.lead_user_id).await?.map(|u| u.name).unwrap_or_default();
        data.push(to_response(t, lead_name));
    }
    Ok(Json(json!({ "data": data })))
}

/// GET /api/v1/competition-teams/mine
#[get("/mine")]
async fn list_mine(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let teams = state.competition_teams.list_for_user(&auth.0.sub).await?;
    let mut data = Vec::with_capacity(teams.len());
    for t in teams {
        let lead_name = state.users.find_by_id(&t.lead_user_id).await?.map(|u| u.name).unwrap_or_default();
        data.push(to_response(t, lead_name));
    }
    Ok(Json(json!({ "data": data })))
}

/// GET /api/v1/competition-teams/<id>
#[get("/<id>")]
async fn get_team(id: String, state: &State<AppState>) -> ApiResult<CompetitionTeamResponse> {
    let team = state.competition_teams.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Team not found".into()))?;
    let lead_name = state.users.find_by_id(&team.lead_user_id).await?.map(|u| u.name).unwrap_or_default();
    Ok(Json(to_response(team, lead_name)))
}

/// POST /api/v1/competition-teams (auth)
#[post("/", data = "<req>")]
async fn create_team(
    req: Json<CreateCompetitionTeamRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<CompetitionTeamResponse> {
    let comp = state.competitions.find_by_id(&req.competition_id).await?
        .ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

    let user = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let max = req.max_members.clamp(2, 10);
    let lead_member = CompetitionTeamMember {
        user_id: user.id.clone(),
        name: user.name.clone(),
        initials: user.initials.clone(),
        role: req.lead_role.clone(),
        joined_at: Utc::now(),
    };
    let mut team = CompetitionTeam::new(
        comp.id.clone(),
        comp.name.clone(),
        req.0.name.clone(),
        req.0.description.clone(),
        user.id.clone(),
        lead_member,
        max,
    );
    team.looking_for = filter_valid_skills(&req.0.looking_for);
    team.open_roles = req.0.open_roles.clone();

    state.competition_teams.create(&team).await?;
    Ok(Json(to_response(team, user.name)))
}

/// PATCH /api/v1/competition-teams/<id>  (lead only)
#[patch("/<id>", data = "<req>")]
async fn update_team(
    id: String,
    req: Json<UpdateCompetitionTeamRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let team = state.competition_teams.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Team not found".into()))?;
    if team.lead_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }
    state.competition_teams.update(&id, &req).await?;
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/v1/competition-teams/<id>  (lead only)
#[delete("/<id>")]
async fn delete_team(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let team = state.competition_teams.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Team not found".into()))?;
    if team.lead_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }
    state.competition_teams.delete(&id).await?;
    Ok(Json(json!({ "success": true })))
}

#[derive(Debug, serde::Deserialize)]
struct ApplyTeamRequest {
    motivation: String,
    role_wanted: Option<String>,
    hours_per_week: Option<String>,
    portfolio_url: Option<String>,
    relevant_experience: Option<String>,
    #[serde(default)]
    availability_start: Option<String>,
    #[serde(default)]
    can_meet_in_person: Option<bool>,
    #[serde(default)]
    additional_links: Vec<String>,
    #[serde(default)]
    comm_channels: Vec<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    agreed_to_coc: bool,
    #[serde(default)]
    skill_confidence: Vec<String>,
}

/// POST /api/v1/competition-teams/<id>/apply  (auth)
#[post("/<id>/apply", data = "<req>")]
async fn apply_to_team(
    id: String,
    req: Json<ApplyTeamRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let team = state.competition_teams.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Team not found".into()))?;
    if team.lead_user_id == auth.0.sub {
        return Err(TeamderError::Conflict("You lead this team".into()).into());
    }
    if team.members.iter().any(|m| m.user_id == auth.0.sub) {
        return Err(TeamderError::Conflict("Already a member".into()).into());
    }
    if team.members.len() >= team.max_members as usize {
        return Err(TeamderError::Conflict("Team is full".into()).into());
    }
    if !matches!(team.status, CompetitionTeamStatus::Recruiting) {
        return Err(TeamderError::Conflict("Team is not recruiting".into()).into());
    }
    if state.join_requests.exists_for_user(&auth.0.sub, &id).await? {
        return Err(TeamderError::Conflict("You already applied".into()).into());
    }
    let from_user = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
    let mut jr = JoinRequest::new(
        &auth.0.sub,
        "competition_team",
        &id,
        &team.name,
        &team.lead_user_id,
        Some(req.motivation.clone()),
    );
    jr.motivation = Some(req.0.motivation.clone());
    jr.role_wanted = req.0.role_wanted.clone();
    jr.hours_per_week = req.0.hours_per_week.clone();
    jr.portfolio_url = req.0.portfolio_url.clone();
    jr.relevant_experience = req.0.relevant_experience.clone();
    jr.availability_start = req.0.availability_start.clone();
    jr.can_meet_in_person = req.0.can_meet_in_person;
    jr.additional_links = req.0.additional_links.clone();
    jr.comm_channels = req.0.comm_channels.clone();
    jr.timezone = req.0.timezone.clone();
    jr.agreed_to_coc = req.0.agreed_to_coc;
    jr.skill_confidence = req.0.skill_confidence.clone();
    state.join_requests.create(&jr).await?;

    let n = Notification::new(
        team.lead_user_id.clone(),
        NotificationKind::JoinRequest,
        "New team application",
        format!("{} wants to join your team {}", from_user.name, team.name),
        Some("/invites".into()),
    );
    let _ = state.notifications.create(&n).await;

    Ok(Json(json!({ "request_id": jr.id })))
}

/// GET /api/v1/competition-teams/<id>/applications  (lead only)
#[get("/<id>/applications")]
async fn list_applications(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let team = state.competition_teams.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Team not found".into()))?;
    if team.lead_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }
    let reqs = state.join_requests.list_pending_for_entity(&id).await?;
    let mut data = Vec::with_capacity(reqs.len());
    for r in reqs {
        let name = state.users.find_by_id(&r.from_user_id).await?
            .map(|u| u.name).unwrap_or_default();
        data.push(JoinRequestResponse {
            id: r.id.clone(),
            from_user_id: r.from_user_id.clone(),
            from_user_name: name,
            entity_type: r.entity_type.clone(),
            entity_id: r.entity_id.clone(),
            entity_name: r.entity_name.clone(),
            owner_id: r.owner_id.clone(),
            message: r.message.clone(),
            status: r.status.clone(),
            motivation: r.motivation.clone(),
            role_wanted: r.role_wanted.clone(),
            hours_per_week: r.hours_per_week.clone(),
            portfolio_url: r.portfolio_url.clone(),
            relevant_experience: r.relevant_experience.clone(),
            availability_start: r.availability_start.clone(),
            can_meet_in_person: r.can_meet_in_person,
            additional_links: r.additional_links.clone(),
            comm_channels: r.comm_channels.clone(),
            timezone: r.timezone.clone(),
            agreed_to_coc: r.agreed_to_coc,
            skill_confidence: r.skill_confidence.clone(),
            created_at: r.created_at,
        });
    }
    Ok(Json(json!({ "data": data })))
}

#[derive(Debug, serde::Deserialize)]
struct RespondTeamApp {
    accept: bool,
    note: Option<String>,
}

/// POST /api/v1/competition-teams/<team_id>/applications/<req_id>/respond  (lead)
#[post("/<team_id>/applications/<req_id>/respond", data = "<body>")]
async fn respond_application(
    team_id: String,
    req_id: String,
    body: Json<RespondTeamApp>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let team = state.competition_teams.find_by_id(&team_id).await?
        .ok_or_else(|| TeamderError::NotFound("Team not found".into()))?;
    if team.lead_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }
    let request = state.join_requests.find_by_id(&req_id).await?
        .ok_or_else(|| TeamderError::NotFound("Application not found".into()))?;
    if request.entity_id != team_id || request.entity_type != "competition_team" {
        return Err(TeamderError::Conflict("Application doesn't belong to this team".into()).into());
    }
    if request.status != JoinRequestStatus::Pending {
        return Err(TeamderError::Conflict("Already responded".into()).into());
    }

    if body.accept {
        if team.members.len() >= team.max_members as usize {
            return Err(TeamderError::Conflict("Team is full".into()).into());
        }
        let user = state.users.find_by_id(&request.from_user_id).await?
            .ok_or_else(|| TeamderError::NotFound("Applicant not found".into()))?;
        let m = CompetitionTeamMember {
            user_id: user.id.clone(),
            name: user.name.clone(),
            initials: user.initials.clone(),
            role: request.role_wanted.clone(),
            joined_at: Utc::now(),
        };
        state.competition_teams.add_member(&team_id, &m).await?;
        // Auto-flip to Full if maxed.
        if team.members.len() + 1 >= team.max_members as usize {
            state.competition_teams.set_status(&team_id, CompetitionTeamStatus::Full).await?;
        }
        state.join_requests.update_status(&req_id, &JoinRequestStatus::Accepted).await?;
    } else {
        state.join_requests.update_status(&req_id, &JoinRequestStatus::Declined).await?;
    }

    let kind = if body.accept { NotificationKind::JoinAccepted } else { NotificationKind::JoinDeclined };
    let title = if body.accept { "Team application accepted" } else { "Team application declined" };
    let body_text = format!(
        "Your application to {} was {}{}",
        team.name,
        if body.accept { "accepted" } else { "declined" },
        body.note.as_ref().map(|s| format!(" — {}", s)).unwrap_or_default()
    );
    let n = Notification::new(
        request.from_user_id.clone(),
        kind,
        title,
        body_text,
        Some(format!("/teams/{}", team_id)),
    );
    let _ = state.notifications.create(&n).await;

    Ok(Json(json!({ "success": true, "status": if body.accept { "accepted" } else { "declined" } })))
}

/// POST /api/v1/competition-teams/<id>/leave  (auth — non-lead members)
#[post("/<id>/leave")]
async fn leave_team(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let team = state.competition_teams.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Team not found".into()))?;
    if team.lead_user_id == auth.0.sub {
        return Err(TeamderError::Conflict("Lead must transfer ownership before leaving".into()).into());
    }
    state.competition_teams.remove_member(&id, &auth.0.sub).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![
        list_teams,
        list_mine,
        get_team,
        create_team,
        update_team,
        delete_team,
        apply_to_team,
        list_applications,
        respond_application,
        leave_team
    ]
}
