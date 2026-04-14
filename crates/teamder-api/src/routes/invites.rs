use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::invite::{Invite, InviteResponse, RespondInviteRequest, SendInviteRequest, InviteStatus},
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// POST /api/v1/invites  (auth)
#[post("/", data = "<req>")]
async fn send_invite(
    req: Json<SendInviteRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<InviteResponse> {
    // Verify recipient exists
    let _ = state
        .users
        .find_by_id(&req.to_user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("User {} not found", req.to_user_id)))?;

    let sender = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Sender not found".into()))?;

    let mut invite = Invite::new(&auth.0.sub, &sender.name, &req.to_user_id);
    invite.message = req.message.clone();

    // Resolve project name if project_id provided
    if let Some(pid) = &req.project_id {
        if let Some(project) = state.projects.find_by_id(pid).await? {
            invite.project_id = Some(pid.clone());
            invite.project_name = Some(project.name);
        }
    }

    state.invites.create(&invite).await?;
    Ok(Json(invite.into()))
}

/// GET /api/v1/invites  (auth — invites for the current user)
#[get("/")]
async fn list_invites(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let invites: Vec<InviteResponse> = state
        .invites
        .list_for_user(&auth.0.sub)
        .await?
        .into_iter()
        .map(InviteResponse::from)
        .collect();

    Ok(Json(json!({ "data": invites })))
}

/// GET /api/v1/invites/<id>  (auth)
#[get("/<id>")]
async fn get_invite(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<InviteResponse> {
    let invite = state
        .invites
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Invite {} not found", id)))?;

    // Only sender or recipient may view
    if invite.from_user_id != auth.0.sub && invite.to_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    Ok(Json(invite.into()))
}

/// POST /api/v1/invites/<id>/respond  (auth — recipient only)
#[post("/<id>/respond", data = "<req>")]
async fn respond_invite(
    id: String,
    req: Json<RespondInviteRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let invite = state
        .invites
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Invite {} not found", id)))?;

    if invite.to_user_id != auth.0.sub {
        return Err(TeamderError::Forbidden.into());
    }

    if invite.status != InviteStatus::Pending {
        return Err(TeamderError::Conflict("Invite is no longer pending".into()).into());
    }

    let new_status = if req.accept {
        InviteStatus::Accepted
    } else {
        InviteStatus::Declined
    };

    state.invites.update_status(&id, &new_status).await?;

    Ok(Json(json!({
        "success": true,
        "status": if req.accept { "accepted" } else { "declined" }
    })))
}

pub fn routes() -> Vec<Route> {
    routes![send_invite, list_invites, get_invite, respond_invite]
}
