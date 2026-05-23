use chrono::Utc;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::notification::Notification;
use teamder_core::models::project_update::ProjectUpdate;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateUpdateBody {
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/projects/<id>/updates")]
pub async fn list_updates(
    state: &State<AppState>,
    id: &str,
) -> Result<Json<Vec<ProjectUpdate>>, ApiError> {
    let updates = state.db.project_update_repo().list_by_project(id).await?;
    Ok(Json(updates))
}

#[rocket::post("/projects/<id>/updates", data = "<body>")]
pub async fn create_update(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<CreateUpdateBody>,
) -> Result<Json<ProjectUpdate>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    // Verify author is a team member
    let is_member = project.team.iter().any(|m| m.user_id == auth.user_id);
    if !is_member {
        return Err(TeamderError::Forbidden("Only team members can post updates".into()).into());
    }

    let author = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let req = body.into_inner();
    let now = Utc::now();
    let update_id = Uuid::new_v4().to_string();

    let update = ProjectUpdate {
        id: update_id.clone(),
        project_id: id.to_string(),
        author_id: auth.user_id.clone(),
        author_name: author.name.clone(),
        kind: req.kind,
        title: req.title.clone(),
        body: req.body.unwrap_or_default(),
        created_at: now,
    };

    state.db.project_update_repo().create(&update).await?;

    // Notify all OTHER team members + lead
    for member in &project.team {
        if member.user_id != auth.user_id {
            let notif = Notification {
                id: Uuid::new_v4().to_string(),
                user_id: member.user_id.clone(),
                kind: "project_update".to_string(),
                title: format!("New update in {}", project.name),
                body: req.title.clone(),
                link: Some(format!("/projects/{}", id)),
                read: false,
                created_at: now,
            };
            let _ = state.db.notification_repo().create(&notif).await;
        }
    }

    Ok(Json(update))
}

#[rocket::delete("/projects/<id>/updates/<update_id>")]
pub async fn delete_update(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    update_id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    // Only author or lead can delete
    let updates = state.db.project_update_repo().list_by_project(id).await?;
    let update = updates
        .iter()
        .find(|u| u.id == update_id)
        .ok_or_else(|| TeamderError::NotFound("Update not found".into()))?;

    if update.author_id != auth.user_id && project.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Not authorized to delete this update".into()).into());
    }

    state.db.project_update_repo().delete(update_id).await?;
    Ok(Json(SuccessResponse { success: true }))
}
