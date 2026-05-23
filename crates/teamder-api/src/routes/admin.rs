use chrono::Utc;
use mongodb::bson;
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

use crate::error::ApiError;
use crate::guards::AdminUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::competition::CompetitionResponse;
use teamder_core::models::project::ProjectResponse;
use teamder_core::models::study_group::StudyGroup;
use teamder_core::models::user::UserResponse;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub users: u64,
    pub projects: u64,
    pub competitions: u64,
    pub study_groups: u64,
}

#[derive(Debug, Serialize)]
pub struct TimeseriesBucket {
    pub date: String,
    pub users: u64,
    pub projects: u64,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct PaginatedUsers {
    pub users: Vec<UserResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: i64,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/admin/stats")]
pub async fn stats(
    state: &State<AppState>,
    _admin: AdminUser,
) -> Result<Json<AdminStats>, ApiError> {
    let users = state.db.user_repo().count().await?;
    let projects = state.db.project_repo().count().await?;
    let competitions = state.db.competition_repo().count().await?;
    let study_groups = state.db.study_group_repo().count().await?;

    Ok(Json(AdminStats {
        users,
        projects,
        competitions,
        study_groups,
    }))
}

#[rocket::get("/admin/timeseries?<range>")]
pub async fn timeseries(
    _state: &State<AppState>,
    _admin: AdminUser,
    range: Option<String>,
) -> Result<Json<Vec<TimeseriesBucket>>, ApiError> {
    let range = range.unwrap_or_else(|| "30d".to_string());
    let days: i64 = match range.as_str() {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        "365d" | "1y" => 365,
        _ => 30,
    };

    // Simple daily-bucketed response
    // In a real implementation, this would use MongoDB aggregation.
    // For now, return a simplified version.
    let mut buckets = Vec::new();
    let now = Utc::now();

    for i in (0..days).rev() {
        let date = now - chrono::Duration::days(i);
        buckets.push(TimeseriesBucket {
            date: date.format("%Y-%m-%d").to_string(),
            users: 0,
            projects: 0,
        });
    }

    Ok(Json(buckets))
}

#[rocket::get("/admin/users?<page>&<limit>")]
pub async fn list_users(
    state: &State<AppState>,
    _admin: AdminUser,
    page: Option<u64>,
    limit: Option<i64>,
) -> Result<Json<PaginatedUsers>, ApiError> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50);
    let skip = (page.saturating_sub(1)) * (limit as u64);

    let (users, total) = state.db.user_repo().list(skip, limit, None).await?;
    let users: Vec<UserResponse> = users.into_iter().map(Into::into).collect();

    Ok(Json(PaginatedUsers {
        users,
        total,
        page,
        limit,
    }))
}

#[rocket::get("/admin/projects")]
pub async fn list_projects(
    state: &State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<ProjectResponse>>, ApiError> {
    let projects = state.db.project_repo().list_all().await?;
    let resp: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

#[rocket::post("/admin/users/<id>/promote")]
pub async fn promote_user(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let user = state
        .db
        .user_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let update = bson::doc! {
        "is_admin": !user.is_admin,
        "updated_at": bson::DateTime::from_chrono(Utc::now()),
    };

    state.db.user_repo().update(id, update).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/admin/projects/<id>/promote")]
pub async fn promote_project(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let project = state
        .db
        .project_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;

    let update = bson::doc! {
        "is_promoted": !project.is_promoted,
        "updated_at": bson::DateTime::from_chrono(Utc::now()),
    };

    state.db.project_repo().update(id, update).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/admin/users/<id>/publisher")]
pub async fn toggle_publisher(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let user = state
        .db
        .user_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let update = bson::doc! {
        "is_publisher": !user.is_publisher,
        "updated_at": bson::DateTime::from_chrono(Utc::now()),
    };

    state.db.user_repo().update(id, update).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::delete("/admin/users/<id>")]
pub async fn delete_user(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    state.db.user_repo().delete(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::delete("/admin/projects/<id>")]
pub async fn delete_project(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    state.db.project_repo().delete(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::get("/admin/export/users.csv")]
pub async fn export_users_csv(
    state: &State<AppState>,
    _admin: AdminUser,
) -> Result<(rocket::http::ContentType, String), ApiError> {
    let (users, _) = state.db.user_repo().list(0, 10000, None).await?;

    let mut csv = String::from("id,name,email,role,department,university,is_admin,created_at\n");
    for u in &users {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            u.id, u.name, u.email, u.role, u.department, u.university, u.is_admin, u.created_at
        ));
    }

    Ok((rocket::http::ContentType::CSV, csv))
}

#[rocket::get("/admin/study-groups")]
pub async fn list_study_groups(
    state: &State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<StudyGroup>>, ApiError> {
    let (groups, _) = state.db.study_group_repo().list(false, 0, 1000).await?;
    Ok(Json(groups))
}

#[rocket::get("/admin/competitions")]
pub async fn list_competitions(
    state: &State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<CompetitionResponse>>, ApiError> {
    let (comps, _) = state
        .db
        .competition_repo()
        .list(None, None, 0, 1000)
        .await?;
    let resp: Vec<CompetitionResponse> = comps
        .into_iter()
        .map(|c| CompetitionResponse::from_competition(c, None, true))
        .collect();
    Ok(Json(resp))
}
