use chrono::Utc;
use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::{
        join_request::{CreateJoinRequestBody, JoinRequest, JoinRequestResponse, JoinRequestStatus, RespondJoinRequestBody},
        project::{JoinMode, TeamMember},
        study_group::GroupMember,
    },
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

fn enrich(req: JoinRequest, from_user_name: String) -> JoinRequestResponse {
    JoinRequestResponse {
        id: req.id,
        from_user_id: req.from_user_id,
        from_user_name,
        entity_type: req.entity_type,
        entity_id: req.entity_id,
        entity_name: req.entity_name,
        owner_id: req.owner_id,
        message: req.message,
        status: req.status,
        motivation: req.motivation,
        role_wanted: req.role_wanted,
        hours_per_week: req.hours_per_week,
        portfolio_url: req.portfolio_url,
        relevant_experience: req.relevant_experience,
        availability_start: req.availability_start,
        can_meet_in_person: req.can_meet_in_person,
        additional_links: req.additional_links,
        comm_channels: req.comm_channels,
        timezone: req.timezone,
        agreed_to_coc: req.agreed_to_coc,
        skill_confidence: req.skill_confidence,
        created_at: req.created_at,
    }
}

/// POST /api/v1/join-requests  (auth)
/// Body: { entity_type: "project"|"study_group", entity_id, message? }
/// If join_mode == Direct → joins immediately and returns { joined: true }
/// If join_mode == Approval → creates pending request
#[post("/", data = "<body>")]
async fn create_request(
    body: Json<CreateJoinRequestBody>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user_id = &auth.0.sub;

    match body.entity_type.as_str() {
        "project" => {
            let project = state.projects.find_by_id(&body.entity_id).await?
                .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

            if project.lead_user_id == *user_id {
                return Err(TeamderError::Conflict("You own this project".into()).into());
            }
            if project.team.iter().any(|m| m.user_id == *user_id) {
                return Err(TeamderError::Conflict("Already a member".into()).into());
            }

            match project.join_mode {
                JoinMode::Direct => {
                    let user = state.users.find_by_id(user_id).await?
                        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
                    let member = TeamMember {
                        user_id: user_id.clone(),
                        initials: user.initials,
                        color: user.gradient,
                        joined_at: Utc::now(),
                    };
                    state.projects.add_member(&body.entity_id, &member).await?;
                    Ok(Json(json!({ "joined": true, "mode": "direct" })))
                }
                JoinMode::Approval => {
                    if state.join_requests.exists_for_user(user_id, &body.entity_id).await? {
                        return Err(TeamderError::Conflict("You already applied".into()).into());
                    }
                    let mut req = JoinRequest::new(
                        user_id,
                        "project",
                        &body.entity_id,
                        &project.name,
                        &project.lead_user_id,
                        body.message.clone(),
                    );
                    req.motivation = body.motivation.clone();
                    req.role_wanted = body.role_wanted.clone();
                    req.hours_per_week = body.hours_per_week.clone();
                    req.portfolio_url = body.portfolio_url.clone();
                    req.relevant_experience = body.relevant_experience.clone();
                    req.availability_start = body.availability_start.clone();
                    req.can_meet_in_person = body.can_meet_in_person;
                    req.additional_links = body.additional_links.clone();
                    req.comm_channels = body.comm_channels.clone();
                    req.timezone = body.timezone.clone();
                    req.agreed_to_coc = body.agreed_to_coc;
                    req.skill_confidence = body.skill_confidence.clone();
                    state.join_requests.create(&req).await?;
                    // Notify project owner
                    let from_user = state.users.find_by_id(user_id).await?;
                    let from_name = from_user.as_ref().map(|u| u.name.clone()).unwrap_or_default();
                    let n = teamder_core::models::notification::Notification::new(
                        project.lead_user_id.clone(),
                        teamder_core::models::notification::NotificationKind::JoinRequest,
                        "New project application",
                        format!("{} wants to join {}", from_name, project.name),
                        Some("/invites".into()),
                    );
                    let _ = state.notifications.create(&n).await;
                    Ok(Json(json!({ "joined": false, "mode": "approval", "request_id": req.id })))
                }
            }
        }
        "study_group" => {
            let group = state.study_groups.find_by_id(&body.entity_id).await?
                .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

            if group.created_by == *user_id {
                return Err(TeamderError::Conflict("You own this group".into()).into());
            }
            if !group.is_open {
                return Err(TeamderError::Conflict("This group is closed".into()).into());
            }
            if group.members.iter().any(|m| m.user_id == *user_id) {
                return Err(TeamderError::Conflict("Already a member".into()).into());
            }
            if group.members.len() >= group.max_members as usize {
                return Err(TeamderError::Conflict("Group is full".into()).into());
            }

            match group.join_mode {
                JoinMode::Direct => {
                    let user = state.users.find_by_id(user_id).await?
                        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
                    let member = GroupMember {
                        user_id: user_id.clone(),
                        initials: user.initials,
                        color: "#4F6D7A".into(),
                        joined_at: Utc::now(),
                        last_checkin: None,
                        streak: 0,
                    };
                    state.study_groups.add_member(&body.entity_id, &member).await?;
                    Ok(Json(json!({ "joined": true, "mode": "direct" })))
                }
                JoinMode::Approval => {
                    if state.join_requests.exists_for_user(user_id, &body.entity_id).await? {
                        return Err(TeamderError::Conflict("You already applied".into()).into());
                    }
                    let mut req = JoinRequest::new(
                        user_id,
                        "study_group",
                        &body.entity_id,
                        &group.name,
                        &group.created_by,
                        body.message.clone(),
                    );
                    req.motivation = body.motivation.clone();
                    state.join_requests.create(&req).await?;
                    Ok(Json(json!({ "joined": false, "mode": "approval", "request_id": req.id })))
                }
            }
        }
        _ => Err(TeamderError::Validation("entity_type must be 'project' or 'study_group'".into()).into()),
    }
}

/// GET /api/v1/join-requests/incoming  (auth)
/// Returns pending requests where current user is the owner.
#[get("/incoming")]
async fn incoming(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let requests = state.join_requests.list_pending_for_owner(&auth.0.sub).await?;
    let user_ids: Vec<&str> = {
        let mut ids: Vec<&str> = requests.iter().map(|r| r.from_user_id.as_str()).collect();
        ids.sort_unstable(); ids.dedup(); ids
    };
    let users = state.users.find_many_by_ids(&user_ids).await?;
    use std::collections::HashMap;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();
    let data: Vec<JoinRequestResponse> = requests.into_iter().map(|r| {
        let name = names.get(r.from_user_id.as_str()).copied().unwrap_or("").to_string();
        enrich(r, name)
    }).collect();
    Ok(Json(json!({ "data": data })))
}

/// GET /api/v1/join-requests/sent  (auth)
/// Returns all requests the current user sent.
#[get("/sent")]
async fn sent(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let user = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
    let requests = state.join_requests.list_by_user(&auth.0.sub).await?;
    let data: Vec<JoinRequestResponse> = requests.into_iter()
        .map(|r| enrich(r, user.name.clone()))
        .collect();
    Ok(Json(json!({ "data": data })))
}

/// POST /api/v1/join-requests/<id>/respond  (auth — must be owner)
/// Body: { accept: bool }
#[post("/<id>/respond", data = "<body>")]
async fn respond(
    id: String,
    body: Json<RespondJoinRequestBody>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let req = state.join_requests.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Request not found".into()))?;

    if req.owner_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }
    if req.status != JoinRequestStatus::Pending {
        return Err(TeamderError::Conflict("Already responded".into()).into());
    }

    let kind = if body.accept {
        teamder_core::models::notification::NotificationKind::JoinAccepted
    } else {
        teamder_core::models::notification::NotificationKind::JoinDeclined
    };
    let title = if body.accept { "Application accepted" } else { "Application declined" };
    let body_text = format!(
        "Your application to {} was {}{}",
        req.entity_name,
        if body.accept { "accepted" } else { "declined" },
        body.note.as_ref().map(|n| format!(" — {}", n)).unwrap_or_default()
    );
    let n = teamder_core::models::notification::Notification::new(
        req.from_user_id.clone(),
        kind,
        title,
        body_text,
        Some(if req.entity_type == "project" {
            format!("/projects")
        } else {
            format!("/study-groups")
        }),
    );
    let _ = state.notifications.create(&n).await;

    if body.accept {
        // Add member to the appropriate entity
        let user = state.users.find_by_id(&req.from_user_id).await?
            .ok_or_else(|| TeamderError::NotFound("Applicant not found".into()))?;

        match req.entity_type.as_str() {
            "project" => {
                let member = TeamMember {
                    user_id: req.from_user_id.clone(),
                    initials: user.initials,
                    color: user.gradient,
                    joined_at: Utc::now(),
                };
                state.projects.add_member(&req.entity_id, &member).await?;
            }
            "study_group" => {
                let member = GroupMember {
                    user_id: req.from_user_id.clone(),
                    initials: user.initials,
                    color: "#4F6D7A".into(),
                    joined_at: Utc::now(),
                    last_checkin: None,
                    streak: 0,
                };
                state.study_groups.add_member(&req.entity_id, &member).await?;
            }
            _ => {}
        }
        state.join_requests.update_status(&id, &JoinRequestStatus::Accepted).await?;
        Ok(Json(json!({ "success": true, "status": "accepted" })))
    } else {
        state.join_requests.update_status(&id, &JoinRequestStatus::Declined).await?;
        Ok(Json(json!({ "success": true, "status": "declined" })))
    }
}

pub fn routes() -> Vec<Route> {
    routes![create_request, incoming, sent, respond]
}
