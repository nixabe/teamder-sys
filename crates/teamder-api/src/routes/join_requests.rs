use chrono::Utc;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::competition_team::CompTeamMember;
use teamder_core::models::join_request::{CreateJoinRequestBody, JoinRequest, JoinRequestResponse};
use teamder_core::models::notification::Notification;
use teamder_core::models::project::TeamMember;
use teamder_core::models::study_group::GroupMember;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RespondBody {
    pub accept: bool,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::post("/join-requests", data = "<body>")]
pub async fn create_join_request(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<CreateJoinRequestBody>,
) -> Result<Json<JoinRequestResponse>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();

    // Determine entity_name and owner_id based on entity_type
    let (entity_name, owner_id) = match req.entity_type.as_str() {
        "project" => {
            let project = state
                .db
                .project_repo()
                .find_by_id(&req.entity_id)
                .await?
                .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;
            (project.name, project.lead_user_id)
        }
        "study_group" => {
            let group = state
                .db
                .study_group_repo()
                .find_by_id(&req.entity_id)
                .await?
                .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;
            (group.name, group.created_by)
        }
        "competition_team" => {
            let team = state
                .db
                .competition_team_repo()
                .find_by_id(&req.entity_id)
                .await?
                .ok_or_else(|| TeamderError::NotFound("Competition team not found".into()))?;
            (team.name, team.lead_user_id)
        }
        _ => {
            return Err(TeamderError::Validation("Invalid entity_type".into()).into());
        }
    };

    // Check for duplicate pending request
    if let Some(_) = state
        .db
        .join_request_repo()
        .find_pending_for_entity(&auth.user_id, &req.entity_id)
        .await?
    {
        return Err(TeamderError::Conflict("A pending request already exists".into()).into());
    }

    let jr = JoinRequest {
        id: Uuid::new_v4().to_string(),
        from_user_id: auth.user_id.clone(),
        entity_type: req.entity_type,
        entity_id: req.entity_id,
        entity_name: entity_name.clone(),
        owner_id: owner_id.clone(),
        message: req.message,
        status: "pending".to_string(),
        motivation: req.motivation,
        role_wanted: req.role_wanted,
        hours_per_week: req.hours_per_week,
        portfolio_url: req.portfolio_url,
        relevant_experience: req.relevant_experience,
        availability_start: req.availability_start,
        can_meet_in_person: req.can_meet_in_person,
        additional_links: req.additional_links.unwrap_or_default(),
        comm_channels: req.comm_channels.unwrap_or_default(),
        timezone: req.timezone,
        agreed_to_coc: req.agreed_to_coc.unwrap_or(false),
        skill_confidence: req.skill_confidence.unwrap_or_default(),
        created_at: now,
    };

    state.db.join_request_repo().create(&jr).await?;

    // Notify owner
    let applicant = state.db.user_repo().find_by_id(&auth.user_id).await?;
    let notif = Notification {
        id: Uuid::new_v4().to_string(),
        user_id: owner_id,
        kind: "join_request".to_string(),
        title: format!(
            "{} wants to join {}",
            applicant.map(|u| u.name).unwrap_or_default(),
            entity_name
        ),
        body: String::new(),
        link: Some("/invites".to_string()),
        read: false,
        created_at: now,
    };
    let _ = state.db.notification_repo().create(&notif).await;

    Ok(Json(jr.into()))
}

#[rocket::get("/join-requests/incoming")]
pub async fn incoming_requests(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<JoinRequestResponse>>, ApiError> {
    let requests = state
        .db
        .join_request_repo()
        .incoming(&auth.user_id)
        .await?;
    let resp: Vec<JoinRequestResponse> = requests.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

#[rocket::get("/join-requests/sent")]
pub async fn sent_requests(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<JoinRequestResponse>>, ApiError> {
    let requests = state.db.join_request_repo().sent(&auth.user_id).await?;
    let resp: Vec<JoinRequestResponse> = requests.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

#[rocket::post("/join-requests/<id>/respond", data = "<body>")]
pub async fn respond_request(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<RespondBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let jr = state
        .db
        .join_request_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Join request not found".into()))?;

    if jr.owner_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the owner can respond".into()).into());
    }

    let new_status = if body.accept { "accepted" } else { "declined" };
    state
        .db
        .join_request_repo()
        .update_status(id, new_status)
        .await?;

    let now = Utc::now();

    if body.accept {
        let user = state
            .db
            .user_repo()
            .find_by_id(&jr.from_user_id)
            .await?
            .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

        match jr.entity_type.as_str() {
            "project" => {
                let member = TeamMember {
                    user_id: jr.from_user_id.clone(),
                    initials: user.initials.clone(),
                    color: user.gradient.clone(),
                    joined_at: now,
                    role: jr.role_wanted.clone(),
                };
                state
                    .db
                    .project_repo()
                    .add_member(&jr.entity_id, &member)
                    .await?;

                // Increment role filled if a role was wanted
                if let Some(role) = &jr.role_wanted {
                    let _ = state
                        .db
                        .project_repo()
                        .increment_role_filled(&jr.entity_id, role)
                        .await;
                }

                // Check auto-activation: all roles filled → recruiting→active
                if let Some(project) = state.db.project_repo().find_by_id(&jr.entity_id).await? {
                    if project.status == "recruiting" {
                        let all_filled = project.roles.iter().all(|r| r.filled >= r.count_needed);
                        if !project.roles.is_empty() && all_filled {
                            let _ = state
                                .db
                                .project_repo()
                                .set_status(&jr.entity_id, "active")
                                .await;
                        }
                    }
                }
            }
            "study_group" => {
                let member = GroupMember {
                    user_id: jr.from_user_id.clone(),
                    initials: user.initials.clone(),
                    color: user.gradient.clone(),
                    joined_at: now,
                    last_checkin: None,
                    streak: 0,
                };
                state
                    .db
                    .study_group_repo()
                    .add_member(&jr.entity_id, &member)
                    .await?;
            }
            "competition_team" => {
                let member = CompTeamMember {
                    user_id: jr.from_user_id.clone(),
                    name: user.name.clone(),
                    initials: user.initials.clone(),
                    role: jr.role_wanted.clone(),
                    joined_at: now,
                };
                state
                    .db
                    .competition_team_repo()
                    .add_member(&jr.entity_id, &member)
                    .await?;

                // If >= max_members auto-set "full"
                if let Some(team) = state
                    .db
                    .competition_team_repo()
                    .find_by_id(&jr.entity_id)
                    .await?
                {
                    if team.members.len() >= team.max_members as usize {
                        let _ = state
                            .db
                            .competition_team_repo()
                            .set_status(&jr.entity_id, "full")
                            .await;
                    }
                }
            }
            _ => {}
        }

        // Notify applicant of acceptance
        let notif = Notification {
            id: Uuid::new_v4().to_string(),
            user_id: jr.from_user_id.clone(),
            kind: "join_accepted".to_string(),
            title: format!("Your request to join {} was accepted!", jr.entity_name),
            body: String::new(),
            link: None,
            read: false,
            created_at: now,
        };
        let _ = state.db.notification_repo().create(&notif).await;
    } else {
        // Notify applicant of decline
        let notif = Notification {
            id: Uuid::new_v4().to_string(),
            user_id: jr.from_user_id.clone(),
            kind: "join_declined".to_string(),
            title: format!("Your request to join {} was declined", jr.entity_name),
            body: body.note.clone().unwrap_or_default(),
            link: None,
            read: false,
            created_at: now,
        };
        let _ = state.db.notification_repo().create(&notif).await;
    }

    Ok(Json(SuccessResponse { success: true }))
}
