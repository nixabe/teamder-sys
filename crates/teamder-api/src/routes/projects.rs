use chrono::Utc;
use mongodb::bson;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::project::{
    CreateProjectRequest, Project, ProjectResponse, TeamMember, UpdateProjectRequest,
};
use teamder_core::models::user::UserResponse;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PaginatedProjects {
    pub projects: Vec<ProjectResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: i64,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetRoleBody {
    pub role: String,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/projects?<page>&<limit>&<status>&<q>")]
pub async fn list_projects(
    state: &State<AppState>,
    page: Option<u64>,
    limit: Option<i64>,
    status: Option<String>,
    q: Option<String>,
) -> Result<Json<PaginatedProjects>, ApiError> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);
    let skip = (page.saturating_sub(1)) * (limit as u64);

    let (projects, total) = state
        .db
        .project_repo()
        .list(skip, limit, status.as_deref(), q.as_deref())
        .await?;

    let projects: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();

    Ok(Json(PaginatedProjects {
        projects,
        total,
        page,
        limit,
    }))
}

#[rocket::get("/projects/my")]
pub async fn my_projects(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ProjectResponse>>, ApiError> {
    let projects = state.db.project_repo().find_by_lead(&auth.user_id).await?;
    let resp: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

#[rocket::get("/projects/joined")]
pub async fn joined_projects(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ProjectResponse>>, ApiError> {
    let projects = state.db.project_repo().find_joined(&auth.user_id).await?;
    let resp: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

#[rocket::get("/projects/<id>")]
pub async fn get_project(
    state: &State<AppState>,
    id: &str,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    Ok(Json(project.into()))
}

#[rocket::post("/projects", data = "<body>")]
pub async fn create_project(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    // Get the lead user for initials
    let lead_user = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let lead_member = TeamMember {
        user_id: auth.user_id.clone(),
        initials: lead_user.initials.clone(),
        color: lead_user.gradient.clone(),
        joined_at: now,
        role: Some("Lead".to_string()),
    };

    let project = Project {
        id: id.clone(),
        name: req.name,
        lead_user_id: auth.user_id,
        icon: req.icon.unwrap_or_else(|| "Pr".to_string()),
        icon_bg: req.icon_bg.unwrap_or_default(),
        status: "recruiting".to_string(),
        description: req.description.unwrap_or_default(),
        goals: req.goals,
        roles: req.roles.unwrap_or_default(),
        skills: req.skills.unwrap_or_default(),
        team: vec![lead_member],
        deadline: req.deadline,
        collab: req.collab,
        duration: req.duration,
        category: req.category,
        is_public: req.is_public.unwrap_or(true),
        join_mode: req.join_mode.unwrap_or_else(|| "direct".to_string()),
        is_promoted: false,
        banner_image: req.banner_image,
        created_at: now,
        updated_at: now,
    };

    state.db.project_repo().create(&project).await?;

    Ok(Json(project.into()))
}

#[rocket::patch("/projects/<id>", data = "<body>")]
pub async fn update_project(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    if project.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the lead can edit this project".into()).into());
    }

    let req = body.into_inner();
    let mut update = bson::doc! {};

    if let Some(v) = &req.name { update.insert("name", v.as_str()); }
    if let Some(v) = &req.description { update.insert("description", v.as_str()); }
    if let Some(v) = &req.goals { update.insert("goals", v.as_str()); }
    if let Some(v) = &req.roles {
        update.insert("roles", bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?);
    }
    if let Some(v) = &req.skills {
        update.insert("skills", bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?);
    }
    if let Some(v) = &req.deadline { update.insert("deadline", v.as_str()); }
    if let Some(v) = &req.collab { update.insert("collab", v.as_str()); }
    if let Some(v) = &req.duration { update.insert("duration", v.as_str()); }
    if let Some(v) = &req.category { update.insert("category", v.as_str()); }
    if let Some(v) = &req.status { update.insert("status", v.as_str()); }
    if let Some(v) = req.is_public { update.insert("is_public", v); }
    if let Some(v) = &req.join_mode { update.insert("join_mode", v.as_str()); }
    if let Some(v) = &req.icon { update.insert("icon", v.as_str()); }
    if let Some(v) = &req.icon_bg { update.insert("icon_bg", v.as_str()); }
    if let Some(v) = req.is_promoted { update.insert("is_promoted", v); }
    if let Some(v) = &req.banner_image { update.insert("banner_image", v.as_str()); }

    update.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));

    state.db.project_repo().update(id, update).await?;

    let updated = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    Ok(Json(updated.into()))
}

#[rocket::delete("/projects/<id>")]
pub async fn delete_project(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    if project.lead_user_id != auth.user_id {
        // Check if admin
        let caller = state.db.user_repo().find_by_id(&auth.user_id).await?;
        if !caller.map(|u| u.is_admin).unwrap_or(false) {
            return Err(
                TeamderError::Forbidden("Only the lead or an admin can delete".into()).into(),
            );
        }
    }

    state.db.project_repo().delete(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::get("/projects/<id>/recommend")]
pub async fn recommend_users(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let _auth = auth; // ensure authenticated
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    let project_skills: std::collections::HashSet<&str> =
        project.skills.iter().map(|s| s.as_str()).collect();

    let (all_users, _) = state.db.user_repo().list(0, 200, None).await?;

    let team_ids: std::collections::HashSet<&str> =
        project.team.iter().map(|m| m.user_id.as_str()).collect();

    let mut scored: Vec<(u8, UserResponse)> = all_users
        .into_iter()
        .filter(|u| !team_ids.contains(u.id.as_str()))
        .filter_map(|u| {
            let user_tags: std::collections::HashSet<&str> =
                u.skill_tags.iter().map(|s| s.as_str()).collect();
            if project_skills.is_empty() || user_tags.is_empty() {
                return None;
            }
            let intersection = project_skills.intersection(&user_tags).count();
            if intersection == 0 {
                return None;
            }
            let union = project_skills.union(&user_tags).count();
            let score = ((intersection as f64 / union as f64) * 100.0) as u8;
            let mut resp: UserResponse = u.into();
            resp.match_score = Some(score);
            Some((score, resp))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let users: Vec<UserResponse> = scored.into_iter().take(20).map(|(_, u)| u).collect();

    Ok(Json(users))
}

#[rocket::post("/projects/<id>/complete")]
pub async fn complete_project(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    if project.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the lead can complete the project".into()).into());
    }

    state.db.project_repo().set_status(id, "completed").await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/projects/<id>/leave")]
pub async fn leave_project(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    if project.lead_user_id == auth.user_id {
        return Err(TeamderError::Validation("Lead cannot leave the project".into()).into());
    }

    // Find the member's role to decrement
    if let Some(member) = project.team.iter().find(|m| m.user_id == auth.user_id) {
        if let Some(role) = &member.role {
            let _ = state.db.project_repo().decrement_role_filled(id, role).await;
        }
    }

    state
        .db
        .project_repo()
        .remove_member(id, &auth.user_id)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/projects/<id>/remove-member/<user_id>")]
pub async fn remove_member(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    user_id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    if project.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the lead can remove members".into()).into());
    }

    if let Some(member) = project.team.iter().find(|m| m.user_id == user_id) {
        if let Some(role) = &member.role {
            let _ = state.db.project_repo().decrement_role_filled(id, role).await;
        }
    }

    state.db.project_repo().remove_member(id, user_id).await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/projects/<id>/set-role/<user_id>", data = "<body>")]
pub async fn set_role(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    user_id: &str,
    body: Json<SetRoleBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    if project.lead_user_id != auth.user_id {
        return Err(TeamderError::Forbidden("Only the lead can set roles".into()).into());
    }

    state
        .db
        .project_repo()
        .set_member_role(id, user_id, &body.role)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}
