use std::collections::HashMap;
use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::invite::{Invite, InviteResponse, RespondInviteRequest, SendInviteRequest, InviteStatus},
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// Resolve names for a batch of invites with a minimum number of lookups.
async fn enrich_invites(
    invites: Vec<Invite>,
    state: &AppState,
) -> Result<Vec<InviteResponse>, TeamderError> {
    // Collect unique user IDs
    let user_ids: Vec<&str> = {
        let mut ids: Vec<&str> = invites.iter()
            .flat_map(|i| [i.from_user_id.as_str(), i.to_user_id.as_str()])
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };
    let users = state.users.find_many_by_ids(&user_ids).await?;
    let user_names: HashMap<&str, &str> = users.iter()
        .map(|u| (u.id.as_str(), u.name.as_str()))
        .collect();

    let mut result = Vec::with_capacity(invites.len());
    for inv in invites {
        let from_user_name = user_names.get(inv.from_user_id.as_str()).copied().unwrap_or("").to_string();
        let to_user_name = user_names.get(inv.to_user_id.as_str()).copied().unwrap_or("").to_string();

        let project_name = if let Some(pid) = &inv.project_id {
            state.projects.find_by_id(pid).await?.map(|p| p.name)
        } else {
            None
        };
        let study_group_name = if let Some(sgid) = &inv.study_group_id {
            state.study_groups.find_by_id(sgid).await?.map(|g| g.name)
        } else {
            None
        };

        result.push(InviteResponse {
            id: inv.id,
            from_user_id: inv.from_user_id,
            from_user_name,
            to_user_id: inv.to_user_id,
            to_user_name,
            project_id: inv.project_id,
            project_name,
            study_group_id: inv.study_group_id,
            study_group_name,
            message: inv.message,
            status: inv.status,
            created_at: inv.created_at,
            expires_at: inv.expires_at,
        });
    }
    Ok(result)
}

/// POST /api/v1/invites  (auth)
#[post("/", data = "<req>")]
async fn send_invite(
    req: Json<SendInviteRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<InviteResponse> {
    // Verify recipient exists and grab their name
    let recipient = state
        .users
        .find_by_id(&req.to_user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("User {} not found", req.to_user_id)))?;

    let sender = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Sender not found".into()))?;

    let mut invite = Invite::new(&auth.0.sub, &req.to_user_id);
    invite.message = req.message.clone();

    if let Some(pid) = &req.project_id {
        if state.projects.find_by_id(pid).await?.is_some() {
            invite.project_id = Some(pid.clone());
        }
    }
    if let Some(sgid) = &req.study_group_id {
        if state.study_groups.find_by_id(sgid).await?.is_some() {
            invite.study_group_id = Some(sgid.clone());
        }
    }

    state.invites.create(&invite).await?;

    // Build full response with resolved names
    let project_name = if let Some(pid) = &invite.project_id {
        state.projects.find_by_id(pid).await?.map(|p| p.name)
    } else {
        None
    };
    let study_group_name = if let Some(sgid) = &invite.study_group_id {
        state.study_groups.find_by_id(sgid).await?.map(|g| g.name)
    } else {
        None
    };

    Ok(Json(InviteResponse {
        id: invite.id,
        from_user_id: invite.from_user_id,
        from_user_name: sender.name,
        to_user_id: invite.to_user_id,
        to_user_name: recipient.name,
        project_id: invite.project_id,
        project_name,
        study_group_id: invite.study_group_id,
        study_group_name,
        message: invite.message,
        status: invite.status,
        created_at: invite.created_at,
        expires_at: invite.expires_at,
    }))
}

/// GET /api/v1/invites  (auth — invites for the current user)
#[get("/")]
async fn list_invites(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let raw = state.invites.list_for_user(&auth.0.sub).await?;
    let invites = enrich_invites(raw, state.inner()).await?;
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

    if invite.from_user_id != auth.0.sub && invite.to_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    let mut enriched = enrich_invites(vec![invite], state.inner()).await?;
    Ok(Json(enriched.remove(0)))
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

    let new_status = if req.accept { InviteStatus::Accepted } else { InviteStatus::Declined };
    state.invites.update_status(&id, &new_status).await?;

    Ok(Json(json!({
        "success": true,
        "status": if req.accept { "accepted" } else { "declined" }
    })))
}

/// DELETE /api/v1/invites/<id>  (auth — sender only)
#[delete("/<id>")]
async fn delete_invite(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let invite = state
        .invites
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Invite {} not found", id)))?;

    if invite.from_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    state.invites.delete_by_id(&id).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![send_invite, list_invites, get_invite, respond_invite, delete_invite]
}
