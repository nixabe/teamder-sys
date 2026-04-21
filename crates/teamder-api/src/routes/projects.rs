use std::collections::HashMap;
use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::project::{CreateProjectRequest, Project, ProjectResponse, UpdateProjectRequest},
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

async fn enrich_projects(
    projects: Vec<Project>,
    state: &AppState,
) -> Result<Vec<ProjectResponse>, TeamderError> {
    let lead_ids: Vec<&str> = {
        let mut ids: Vec<&str> = projects.iter().map(|p| p.lead_user_id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };
    let users = state.users.find_many_by_ids(&lead_ids).await?;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();

    Ok(projects
        .into_iter()
        .map(|p| {
            let lead_name = names.get(p.lead_user_id.as_str()).copied().unwrap_or("").to_string();
            ProjectResponse::from_project(p, lead_name)
        })
        .collect())
}

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

    let raw = if let Some(query) = q {
        state.projects.search(&query).await?
    } else if let Some(s) = status {
        state.projects.list_by_status(&s).await?
    } else {
        state.projects.list(limit, skip).await?
    };

    let total = state.projects.count().await?;
    let projects = enrich_projects(raw, state.inner()).await?;

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
    let lead_name = state.users.find_by_id(&project.lead_user_id).await?
        .map(|u| u.name).unwrap_or_default();
    Ok(Json(ProjectResponse::from_project(project, lead_name)))
}

/// POST /api/v1/projects  (requires auth)
#[post("/", data = "<req>")]
async fn create_project(
    req: Json<CreateProjectRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<ProjectResponse> {
    let lead_name = state.users.find_by_id(&auth.0.sub).await?
        .map(|u| u.name)
        .unwrap_or_else(|| "Unknown".into());

    let mut project = Project::new(&req.name, &auth.0.sub, &req.description);

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

    Ok(Json(ProjectResponse::from_project(project, lead_name)))
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
    let raw = state.projects.list_by_lead(&auth.0.sub).await?;
    let lead_name = state.users.find_by_id(&auth.0.sub).await?
        .map(|u| u.name).unwrap_or_default();
    let projects: Vec<ProjectResponse> = raw
        .into_iter()
        .map(|p| ProjectResponse::from_project(p, lead_name.clone()))
        .collect();
    Ok(Json(json!({ "data": projects })))
}

pub fn routes() -> Vec<Route> {
    routes![list_projects, get_project, create_project, update_project, delete_project, my_projects]
}
