use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};

use crate::{error::ApiResult, guards::AdminUser, state::AppState};

/// GET /api/v1/admin/stats  (admin only)
/// Returns high-level platform statistics for the admin dashboard.
#[get("/stats")]
async fn stats(_admin: AdminUser, state: &State<AppState>) -> ApiResult<Value> {
    let (users, projects, competitions, groups) = tokio::join!(
        state.users.count(),
        state.projects.count(),
        state.competitions.count(),
        state.study_groups.count(),
    );

    Ok(Json(json!({
        "users":        users?,
        "projects":     projects?,
        "competitions": competitions?,
        "study_groups": groups?,
    })))
}

/// GET /api/v1/admin/users  (admin only — full user list, higher limit)
#[get("/users?<limit>&<skip>")]
async fn list_users(
    limit: Option<i64>,
    skip: Option<u64>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    use teamder_core::models::user::UserResponse;

    let limit = limit.unwrap_or(50).min(200);
    let skip = skip.unwrap_or(0);

    let users: Vec<UserResponse> = state
        .users
        .list(limit, skip)
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    let total = state.users.count().await?;

    Ok(Json(json!({
        "data": users,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

/// GET /api/v1/admin/projects  (admin only)
#[get("/projects?<limit>&<skip>")]
async fn list_projects(
    limit: Option<i64>,
    skip: Option<u64>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    use teamder_core::models::project::ProjectResponse;

    let limit = limit.unwrap_or(50).min(200);
    let skip = skip.unwrap_or(0);

    let projects: Vec<ProjectResponse> = state
        .projects
        .list(limit, skip)
        .await?
        .into_iter()
        .map(ProjectResponse::from)
        .collect();

    let total = state.projects.count().await?;

    Ok(Json(json!({
        "data": projects,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

pub fn routes() -> Vec<Route> {
    routes![stats, list_users, list_projects]
}
