use std::collections::HashMap;
use rocket::{Route, State, response::content::RawText, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{error::TeamderError, models::user::User};

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

    let raw = state.projects.list(limit, skip).await?;
    let lead_ids: Vec<&str> = {
        let mut ids: Vec<&str> = raw.iter().map(|p| p.lead_user_id.as_str()).collect();
        ids.sort_unstable(); ids.dedup(); ids
    };
    let users = state.users.find_many_by_ids(&lead_ids).await?;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();
    let projects: Vec<ProjectResponse> = raw.into_iter().map(|p| {
        let lead_name = names.get(p.lead_user_id.as_str()).copied().unwrap_or("").to_string();
        ProjectResponse::from_project(p, lead_name)
    }).collect();

    let total = state.projects.count().await?;

    Ok(Json(json!({
        "data": projects,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

/// POST /api/v1/admin/users/<id>/promote  (admin only)
#[post("/users/<id>/promote", data = "<req>")]
async fn promote_user(
    id: String,
    req: Json<Value>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let val = req.0.get("value").and_then(|v| v.as_bool()).unwrap_or(true);
    state.users.set_admin(&id, val).await?;
    Ok(Json(json!({ "success": true, "is_admin": val })))
}

/// POST /api/v1/admin/projects/<id>/promote  (admin only)
#[post("/projects/<id>/promote", data = "<req>")]
async fn promote_project(
    id: String,
    req: Json<Value>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let val = req.0.get("value").and_then(|v| v.as_bool()).unwrap_or(true);
    state.projects.set_promoted(&id, val).await?;
    Ok(Json(json!({ "success": true, "is_promoted": val })))
}

/// GET /api/v1/admin/timeseries?range=30d
///
/// Buckets users created over a time window into daily counts.
/// `range` accepts: 7d, 30d, 90d, 365d.
#[get("/timeseries?<range>")]
async fn timeseries(
    range: Option<String>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    use chrono::{Datelike, Duration, Utc};

    let days: i64 = match range.as_deref().unwrap_or("30d") {
        "7d" => 7,
        "90d" => 90,
        "365d" | "1y" => 365,
        _ => 30,
    };

    let now = Utc::now();
    let since = now - Duration::days(days);

    // Pull all users (could be a lot — admin only).
    let all_users: Vec<User> = state.users.list(2000, 0).await?;
    let all_projects = state.projects.list(2000, 0).await?;

    let mut user_buckets: HashMap<String, u32> = HashMap::new();
    let mut project_buckets: HashMap<String, u32> = HashMap::new();

    for u in &all_users {
        if u.created_at >= since {
            let key = format!("{:04}-{:02}-{:02}", u.created_at.year(), u.created_at.month(), u.created_at.day());
            *user_buckets.entry(key).or_insert(0) += 1;
        }
    }
    for p in &all_projects {
        if p.created_at >= since {
            let key = format!("{:04}-{:02}-{:02}", p.created_at.year(), p.created_at.month(), p.created_at.day());
            *project_buckets.entry(key).or_insert(0) += 1;
        }
    }

    // Build a sorted series of all days in the range.
    let mut series = Vec::new();
    for d in 0..=days {
        let day = since + Duration::days(d);
        let key = format!("{:04}-{:02}-{:02}", day.year(), day.month(), day.day());
        series.push(json!({
            "date": key,
            "users": *user_buckets.get(&key).unwrap_or(&0),
            "projects": *project_buckets.get(&key).unwrap_or(&0),
        }));
    }

    // Match success rate = projects that left "Recruiting" status / total projects (last `days`).
    let recent_projects: Vec<_> = all_projects.iter().filter(|p| p.created_at >= since).collect();
    let recent_total = recent_projects.len();
    let matched = recent_projects
        .iter()
        .filter(|p| !matches!(p.status, teamder_core::models::project::ProjectStatus::Recruiting))
        .count();
    let success_rate = if recent_total > 0 {
        (matched as f64 / recent_total as f64) * 100.0
    } else {
        0.0
    };

    // DAU proxy: users who updated their profile within the day.
    let dau_today = all_users
        .iter()
        .filter(|u| (now - u.updated_at).num_hours() < 24)
        .count();
    let mau = all_users
        .iter()
        .filter(|u| (now - u.updated_at).num_days() < 30)
        .count();

    Ok(Json(json!({
        "range": format!("{}d", days),
        "series": series,
        "match_success_rate": success_rate,
        "dau": dau_today,
        "mau": mau,
        "total_users": all_users.len(),
        "total_projects": all_projects.len(),
    })))
}

/// GET /api/v1/admin/export/users.csv  — CSV export of users (admin only).
#[get("/export/users.csv")]
async fn export_users_csv(_admin: AdminUser, state: &State<AppState>) -> Result<RawText<String>, crate::error::ApiError> {
    let users = state.users.list(5000, 0).await
        .map_err(crate::error::ApiError::from)?;

    let mut out = String::from("id,email,name,role,department,year,location,projects_done,rating,created_at\n");
    for u in users {
        let esc = |s: &str| s.replace('"', "\"\"");
        out.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\"\n",
            esc(&u.id),
            esc(&u.email),
            esc(&u.name),
            esc(&u.role),
            esc(&u.department),
            esc(&u.year),
            esc(u.location.as_deref().unwrap_or("")),
            u.projects_done,
            u.rating,
            u.created_at.to_rfc3339(),
        ));
    }
    Ok(RawText(out))
}

#[allow(dead_code)]
fn _silence(_: TeamderError) {}

pub fn routes() -> Vec<Route> {
    routes![stats, list_users, list_projects, promote_project, promote_user, timeseries, export_users_csv]
}
