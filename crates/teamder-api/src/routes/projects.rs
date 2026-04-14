use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::project::{CreateProjectRequest, Project, ProjectResponse, UpdateProjectRequest},
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// GET /api/v1/projects?limit=20&skip=0&status=recruiting&q=query
#[get("/?<limit>&<skip>&<status>&<q>")]
async fn list_projects(
    limit: Option<i64>,
    skip: Option<u64>,
    status: Option<String>,
    q: Option<String>,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let limit = limit.unwrap_or(20).min(100);
    let skip = skip.unwrap_or(0);

    let projects: Vec<ProjectResponse> = if let Some(query) = q {
        state
            .projects
            .search(&query)
            .await?
            .into_iter()
            .map(ProjectResponse::from)
            .collect()
    } else if let Some(s) = status {
        state
            .projects
            .list_by_status(&s)
            .await?
            .into_iter()
            .map(ProjectResponse::from)
            .collect()
    } else {
        state
            .projects
            .list(limit, skip)
            .await?
            .into_iter()
            .map(ProjectResponse::from)
            .collect()
    };

    let total = state.projects.count().await?;

    Ok(Json(json!({
        "data": projects,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

/// GET /api/v1/projects/<id>
#[get("/<id>")]
async fn get_project(id: String, state: &State<AppState>) -> ApiResult<ProjectResponse> {
    let project = state
        .projects
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Project {} not found", id)))?;
    Ok(Json(project.into()))
}

/// POST /api/v1/projects  (requires auth)
#[post("/", data = "<req>")]
async fn create_project(
    req: Json<CreateProjectRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<ProjectResponse> {
    // Resolve author name
    let lead_name = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .map(|u| u.name)
        .unwrap_or_else(|| "Unknown".into());

    let mut project = Project::new(
        &req.name,
        &auth.0.sub,
        lead_name,
        &req.description,
    );

    // Apply optional fields
    if let Some(v) = &req.goals { project.goals = Some(v.clone()); }
    if let Some(v) = &req.roles { project.roles = v.clone(); }
    project.skills = req.skills.clone();
    if let Some(v) = &req.deadline { project.deadline = Some(v.clone()); }
    project.collab = req.collab.clone();
    if let Some(v) = &req.duration { project.duration = Some(v.clone()); }
    if let Some(v) = &req.category { project.category = Some(v.clone()); }
    if let Some(v) = req.is_public { project.is_public = v; }
    if let Some(v) = &req.icon { project.icon = v.clone(); }
    if let Some(v) = &req.icon_bg { project.icon_bg = v.clone(); }

    state.projects.create(&project).await?;

    Ok(Json(project.into()))
}

/// PATCH /api/v1/projects/<id>  (auth + owner or admin)
#[patch("/<id>", data = "<req>")]
async fn update_project(
    id: String,
    req: Json<UpdateProjectRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let project = state
        .projects
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Project {} not found", id)))?;

    if project.lead_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    state.projects.update(&id, &req).await?;

    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/v1/projects/<id>  (auth + owner or admin)
#[delete("/<id>")]
async fn delete_project(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let project = state
        .projects
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Project {} not found", id)))?;

    if project.lead_user_id != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    state.projects.delete(&id).await?;

    Ok(Json(json!({ "success": true })))
}

/// GET /api/v1/projects/my  (auth — projects led by current user)
#[get("/my")]
async fn my_projects(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let projects: Vec<ProjectResponse> = state
        .projects
        .list_by_lead(&auth.0.sub)
        .await?
        .into_iter()
        .map(ProjectResponse::from)
        .collect();

    Ok(Json(json!({ "data": projects })))
}

pub fn routes() -> Vec<Route> {
    routes![list_projects, get_project, create_project, update_project, delete_project, my_projects]
}
