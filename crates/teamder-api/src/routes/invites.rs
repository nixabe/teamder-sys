use chrono::{Duration, Utc};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::competition_team::CompTeamMember;
use teamder_core::models::invite::Invite;
use teamder_core::models::notification::Notification;
use teamder_core::models::project::TeamMember;
use teamder_core::models::study_group::GroupMember;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SendInviteBody {
    pub to_user_id: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub study_group_id: Option<String>,
    #[serde(default)]
    pub competition_team_id: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RespondBody {
    pub accept: bool,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/invites")]
pub async fn list_invites(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Invite>>, ApiError> {
    let invites = state
        .db
        .invite_repo()
        .list_for_user(&auth.user_id)
        .await?;
    Ok(Json(invites))
}

#[rocket::get("/invites/<id>")]
pub async fn get_invite(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<Invite>, ApiError> {
    let invite = state
        .db
        .invite_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Invite not found".into()))?;

    if invite.from_user_id != auth.user_id && invite.to_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Not authorized".into()).into());
    }

    Ok(Json(invite))
}

#[rocket::post("/invites", data = "<body>")]
pub async fn send_invite(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<SendInviteBody>,
) -> Result<Json<Invite>, ApiError> {
    let req = body.into_inner();

    // Check duplicate pending invite
    let existing = state
        .db
        .invite_repo()
        .find_pending_between(
            &auth.user_id,
            &req.to_user_id,
            req.project_id.as_deref(),
            req.study_group_id.as_deref(),
            req.competition_team_id.as_deref(),
        )
        .await?;

    if existing.is_some() {
        return Err(TeamderError::Conflict("A pending invite already exists".into()).into());
    }

    let now = Utc::now();
    let expires_at = now + Duration::days(7);
    let id = Uuid::new_v4().to_string();

    let invite = Invite {
        id: id.clone(),
        from_user_id: auth.user_id.clone(),
        to_user_id: req.to_user_id.clone(),
        project_id: req.project_id,
        study_group_id: req.study_group_id,
        competition_team_id: req.competition_team_id,
        message: req.message,
        status: "pending".to_string(),
        is_read: false,
        created_at: now,
        expires_at,
    };

    state.db.invite_repo().create(&invite).await?;

    // Create notification for recipient
    let sender = state.db.user_repo().find_by_id(&auth.user_id).await?;
    let notif = Notification {
        id: Uuid::new_v4().to_string(),
        user_id: req.to_user_id,
        kind: "invite".to_string(),
        title: format!(
            "{} sent you an invite",
            sender.map(|u| u.name).unwrap_or_default()
        ),
        body: String::new(),
        link: Some("/invites".to_string()),
        read: false,
        created_at: now,
    };
    let _ = state.db.notification_repo().create(&notif).await;

    Ok(Json(invite))
}

#[rocket::post("/invites/<id>/respond", data = "<body>")]
pub async fn respond_invite(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<RespondBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let invite = state
        .db
        .invite_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Invite not found".into()))?;

    if invite.to_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the recipient can respond".into()).into());
    }

    let new_status = if body.accept { "accepted" } else { "declined" };
    state
        .db
        .invite_repo()
        .update_status(id, new_status)
        .await?;

    // On accept, add user to the referenced entity
    if body.accept {
        let now = Utc::now();
        let user = state
            .db
            .user_repo()
            .find_by_id(&auth.user_id)
            .await?
            .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

        if let Some(project_id) = &invite.project_id {
            let member = TeamMember {
                user_id: auth.user_id.clone(),
                initials: user.initials.clone(),
                color: user.gradient.clone(),
                joined_at: now,
                role: None,
            };
            let _ = state
                .db
                .project_repo()
                .add_member(project_id, &member)
                .await;
        }

        if let Some(sg_id) = &invite.study_group_id {
            let member = GroupMember {
                user_id: auth.user_id.clone(),
                initials: user.initials.clone(),
                color: user.gradient.clone(),
                joined_at: now,
                last_checkin: None,
                streak: 0,
            };
            let _ = state
                .db
                .study_group_repo()
                .add_member(sg_id, &member)
                .await;
        }

        if let Some(ct_id) = &invite.competition_team_id {
            let member = CompTeamMember {
                user_id: auth.user_id.clone(),
                name: user.name.clone(),
                initials: user.initials.clone(),
                role: None,
                joined_at: now,
            };
            let _ = state
                .db
                .competition_team_repo()
                .add_member(ct_id, &member)
                .await;
        }
    }

    // Create notification for sender
    let kind = if body.accept {
        "invite_accepted"
    } else {
        "invite_declined"
    };
    let responder = state.db.user_repo().find_by_id(&auth.user_id).await?;
    let notif = Notification {
        id: Uuid::new_v4().to_string(),
        user_id: invite.from_user_id,
        kind: kind.to_string(),
        title: format!(
            "{} {} your invite",
            responder.map(|u| u.name).unwrap_or_default(),
            if body.accept { "accepted" } else { "declined" }
        ),
        body: String::new(),
        link: Some("/invites".to_string()),
        read: false,
        created_at: Utc::now(),
    };
    let _ = state.db.notification_repo().create(&notif).await;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::patch("/invites/<id>/read")]
pub async fn mark_read(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let _auth = auth; // ensure authenticated
    state.db.invite_repo().mark_read(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/invites/read-all")]
pub async fn read_all(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<SuccessResponse>, ApiError> {
    state
        .db
        .invite_repo()
        .mark_all_read(&auth.user_id)
        .await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::delete("/invites/<id>")]
pub async fn delete_invite(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let invite = state
        .db
        .invite_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Invite not found".into()))?;

    if invite.from_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the sender can delete".into()).into());
    }

    state.db.invite_repo().delete(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}
