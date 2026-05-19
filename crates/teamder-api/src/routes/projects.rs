use std::collections::HashMap;
use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::notification::{Notification, NotificationKind},
    models::project::{CreateProjectRequest, Project, ProjectDetail, ProjectResponse, ProjectStatus, TeamMemberEnriched, UpdateProjectRequest},
    models::user::UserResponse,
    skills::compute_project_match_score,
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// After a member is added or a role is assigned, check if all roles are now filled.
/// If yes and the project is still Recruiting, auto-transition it to Active.
pub async fn maybe_activate_project(project_id: &str, state: &AppState) {
    let Ok(Some(p)) = state.projects.find_by_id(project_id).await else { return };
    if p.status != ProjectStatus::Recruiting { return }
    if p.roles.is_empty() { return }
    let all_filled = p.roles.iter().all(|r| r.filled >= r.count_needed);
    if !all_filled { return }
    let update = UpdateProjectRequest {
        name: None, description: None, goals: None,
        status: Some(ProjectStatus::Active),
        roles: None, skills: None, deadline: None,
        collab: None, duration: None, is_public: None,
        join_mode: None, banner_image: None,
    };
    let _ = state.projects.update(project_id, &update).await;
}

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

    let (raw, total) = if let Some(query) = q {
        let r = state.projects.search(&query).await?;
        let t = r.len() as u64;
        (r, t)
    } else if let Some(ref s) = status {
        let r = state.projects.list_by_status(s, limit, skip).await?;
        let t = state.projects.count_by_status(s).await?;
        (r, t)
    } else {
        let r = state.projects.list_with_promotion(limit, skip).await?;
        let t = state.projects.count().await?;
        (r, t)
    };

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
    if let Some(v) = req.join_mode.clone() { project.join_mode = v; }
    if req.banner_image.is_some() { project.banner_image = req.banner_image.clone(); }

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

/// GET /api/v1/projects/my  (auth — projects led by current user, enriched
/// like /joined so the My Teams page can render them with the same card.)
#[get("/my")]
async fn my_projects(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let raw = state.projects.list_by_lead(&auth.0.sub).await?;

    let mut all_ids: Vec<&str> = raw.iter()
        .flat_map(|p| std::iter::once(p.lead_user_id.as_str())
            .chain(p.team.iter().map(|m| m.user_id.as_str())))
        .collect();
    all_ids.sort_unstable(); all_ids.dedup();
    let users = state.users.find_many_by_ids(&all_ids).await?;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();

    let data: Vec<ProjectDetail> = raw.into_iter().map(|p| {
        let lead_name = names.get(p.lead_user_id.as_str()).copied().unwrap_or("").to_string();
        let team: Vec<TeamMemberEnriched> = p.team.iter().map(|m| TeamMemberEnriched {
            user_id: m.user_id.clone(),
            name: names.get(m.user_id.as_str()).copied().unwrap_or("").to_string(),
            initials: m.initials.clone(),
            color: m.color.clone(),
            joined_at: m.joined_at,
        }).collect();
        ProjectDetail {
            id: p.id, name: p.name, lead_user_id: p.lead_user_id, lead_name,
            icon: p.icon, icon_bg: p.icon_bg, status: p.status,
            description: p.description, goals: p.goals, roles: p.roles,
            skills: p.skills, team, deadline: p.deadline, collab: p.collab,
            duration: p.duration, category: p.category, is_public: p.is_public,
            join_mode: p.join_mode, is_promoted: p.is_promoted, banner_image: p.banner_image, created_at: p.created_at,
        }
    }).collect();

    Ok(Json(json!({ "data": data })))
}

/// GET /api/v1/projects/joined  (auth — projects where current user is a team member)
#[get("/joined")]
async fn joined_projects(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let user_id = &auth.0.sub;
    let raw = state.projects.list_by_member(user_id).await?;

    // Collect all unique user_ids from all teams + leads
    let mut all_ids: Vec<&str> = raw.iter()
        .flat_map(|p| std::iter::once(p.lead_user_id.as_str())
            .chain(p.team.iter().map(|m| m.user_id.as_str())))
        .collect();
    all_ids.sort_unstable(); all_ids.dedup();
    let users = state.users.find_many_by_ids(&all_ids).await?;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();

    let data: Vec<ProjectDetail> = raw.into_iter().map(|p| {
        let lead_name = names.get(p.lead_user_id.as_str()).copied().unwrap_or("").to_string();
        let team: Vec<TeamMemberEnriched> = p.team.iter().map(|m| TeamMemberEnriched {
            user_id: m.user_id.clone(),
            name: names.get(m.user_id.as_str()).copied().unwrap_or("").to_string(),
            initials: m.initials.clone(),
            color: m.color.clone(),
            joined_at: m.joined_at,
        }).collect();
        ProjectDetail {
            id: p.id, name: p.name, lead_user_id: p.lead_user_id, lead_name,
            icon: p.icon, icon_bg: p.icon_bg, status: p.status,
            description: p.description, goals: p.goals, roles: p.roles,
            skills: p.skills, team, deadline: p.deadline, collab: p.collab,
            duration: p.duration, category: p.category, is_public: p.is_public,
            join_mode: p.join_mode, is_promoted: p.is_promoted, banner_image: p.banner_image, created_at: p.created_at,
        }
    }).collect();

    Ok(Json(json!({ "data": data })))
}

/// GET /api/v1/projects/<id>/recommend?limit=10 — suggested teammates.
///
/// Returns up to `limit` users (default 10) ranked by `compute_project_match_score`
/// against the project's required skills, excluding the lead and current team
/// members. Available to anyone who can read the project.
#[get("/<id>/recommend?<limit>")]
async fn recommend_users(
    id: String,
    limit: Option<i64>,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let project = state
        .projects
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Project {} not found", id)))?;

    let limit = limit.unwrap_or(10).clamp(1, 50);
    let pool = state.users.list(200, 0).await?;

    let mut excluded: std::collections::HashSet<String> =
        project.team.iter().map(|m| m.user_id.clone()).collect();
    excluded.insert(project.lead_user_id.clone());

    let mut scored: Vec<(u8, teamder_core::models::user::User)> = pool
        .into_iter()
        .filter(|u| !excluded.contains(&u.id) && u.is_public)
        .map(|u| (compute_project_match_score(&project, &u), u))
        .collect();

    // Sort by score descending; tie-break on rating then projects_done.
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.rating.partial_cmp(&a.1.rating).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| b.1.projects_done.cmp(&a.1.projects_done))
    });

    let data: Vec<Value> = scored
        .into_iter()
        .take(limit as usize)
        .map(|(score, u)| {
            let mut resp: UserResponse = u.into();
            resp.match_score = score;
            json!(resp)
        })
        .collect();

    Ok(Json(json!({ "data": data, "project_id": project.id, "project_name": project.name })))
}

/// DELETE /api/v1/projects/<id>/members/<user_id>  (auth + owner or admin)
#[delete("/<id>/members/<user_id>")]
async fn remove_member(
    id: String,
    user_id: String,
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
    // Prevent removing the lead themselves
    if user_id == project.lead_user_id {
        return Err(TeamderError::Validation("Cannot remove the project lead".into()).into());
    }

    state.projects.remove_member(&id, &user_id).await?;
    Ok(Json(json!({ "success": true })))
}

/// PATCH /api/v1/projects/<id>/members/<user_id>/role  (auth + owner or admin)
#[derive(serde::Deserialize)]
struct SetMemberRoleBody {
    role: Option<String>,
}

#[patch("/<id>/members/<user_id>/role", data = "<body>")]
async fn set_member_role(
    id: String,
    user_id: String,
    body: Json<SetMemberRoleBody>,
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

    state.projects.set_member_role(&id, &user_id, body.role.as_deref()).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/projects/<id>/complete  (auth + owner or admin)
#[post("/<id>/complete")]
async fn complete_project(
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

    if project.status == ProjectStatus::Completed {
        return Err(TeamderError::Conflict("Project is already completed".into()).into());
    }

    let update = UpdateProjectRequest {
        name: None, description: None, goals: None,
        status: Some(ProjectStatus::Completed),
        roles: None, skills: None, deadline: None,
        collab: None, duration: None, is_public: None,
        join_mode: None, banner_image: None,
    };
    state.projects.update(&id, &update).await?;

    let lead_name = state.users.find_by_id(&project.lead_user_id).await?
        .map(|u| u.name).unwrap_or_default();

    for member in &project.team {
        let n = Notification::new(
            &member.user_id,
            NotificationKind::System,
            format!("{} is completed!", project.name),
            format!("{} marked \"{}\" as completed. You can now leave reviews for your teammates.", lead_name, project.name),
            Some(format!("/projects/{}", project.id)),
        );
        if let Err(e) = state.notifications.create(&n).await {
            tracing::warn!("failed to create completion notification: {e}");
        }
    }

    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/projects/<id>/leave  (auth — member only, not lead)
#[post("/<id>/leave")]
async fn leave_project(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user_id = &auth.0.sub;
    let project = state.projects.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound(format!("Project {} not found", id)))?;

    if project.lead_user_id == *user_id {
        return Err(TeamderError::Validation("Project lead cannot leave — transfer ownership or delete the project instead".into()).into());
    }
    if !project.team.iter().any(|m| m.user_id == *user_id) {
        return Err(TeamderError::NotFound("You are not a member of this project".into()).into());
    }

    // 5-minute cooldown check
    let log_key = format!("{}:{}", user_id, id);
    {
        let log = state.leave_log.lock().await;
        if let Some(left_at) = log.get(&log_key) {
            let elapsed = chrono::Utc::now() - *left_at;
            if elapsed.num_seconds() < 300 {
                let remaining = 300 - elapsed.num_seconds();
                return Err(TeamderError::Conflict(
                    format!("You recently left this project. Please wait {} more seconds before rejoining is allowed.", remaining)
                ).into());
            }
        }
    }

    // Decrement filled count for the role the member held
    let member_role = project.team.iter()
        .find(|m| m.user_id == *user_id)
        .and_then(|m| m.role.clone());
    if let Some(role_name) = &member_role {
        let _ = state.projects.increment_role_filled(&id, role_name, -1).await;
    }

    state.projects.remove_member(&id, user_id).await?;

    // Record leave time for cooldown
    {
        let mut log = state.leave_log.lock().await;
        log.insert(log_key, chrono::Utc::now());
    }

    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_projects, get_project, create_project, update_project, delete_project, my_projects, joined_projects, recommend_users, remove_member, set_member_role, complete_project, leave_project]
}
