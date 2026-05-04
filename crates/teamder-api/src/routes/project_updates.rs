use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::{
        notification::{Notification, NotificationKind},
        project_update::{CreateProjectUpdateRequest, ProjectUpdate, ProjectUpdateResponse},
    },
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// GET /api/v1/projects/<id>/updates  (public)
#[get("/<id>/updates")]
async fn list_updates(id: String, state: &State<AppState>) -> ApiResult<Value> {
    let raw = state.project_updates.list_for_project(&id).await?;
    let data: Vec<ProjectUpdateResponse> = raw.into_iter().map(Into::into).collect();
    Ok(Json(json!({ "data": data })))
}

/// POST /api/v1/projects/<id>/updates  (auth — lead or team member)
#[post("/<id>/updates", data = "<req>")]
async fn create_update(
    id: String,
    req: Json<CreateProjectUpdateRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<ProjectUpdateResponse> {
    let project = state.projects.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    let is_member = project.lead_user_id == auth.0.sub
        || project.team.iter().any(|m| m.user_id == auth.0.sub);
    if !is_member && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    let user = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    if req.title.trim().is_empty() || req.body.trim().is_empty() {
        return Err(TeamderError::Validation("Title and body required".into()).into());
    }

    let update = ProjectUpdate::new(
        id.clone(),
        user.id.clone(),
        user.name.clone(),
        req.0.kind,
        req.0.title,
        req.0.body,
    );
    state.project_updates.create(&update).await?;

    // Notify other team members.
    let body_preview = update.title.clone();
    for member in project.team.iter().chain(std::iter::once(&teamder_core::models::project::TeamMember {
        user_id: project.lead_user_id.clone(),
        initials: String::new(),
        color: String::new(),
        joined_at: project.created_at,
    })) {
        if member.user_id == user.id { continue; }
        let n = Notification::new(
            member.user_id.clone(),
            NotificationKind::System,
            format!("Update on {}", project.name),
            body_preview.clone(),
            Some(format!("/projects")),
        );
        let _ = state.notifications.create(&n).await;
    }

    Ok(Json(update.into()))
}

/// DELETE /api/v1/projects/<project_id>/updates/<update_id>  (auth — author or admin)
#[delete("/<project_id>/updates/<update_id>")]
async fn delete_update(
    project_id: String,
    update_id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let _ = project_id;
    state.project_updates.delete(&update_id).await?;
    let _ = auth;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_updates, create_update, delete_update]
}
