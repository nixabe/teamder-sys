use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::models::{
    competition::CompetitionResponse,
    project::ProjectResponse,
    study_group::StudyGroup,
    user::UserResponse,
};

use crate::{error::ApiResult, state::AppState};

/// GET /api/v1/search?q=…
///
/// Cross-entity search: returns up to 10 hits each from users, projects,
/// competitions, and study groups. Used by the global search page.
#[get("/?<q>")]
async fn search_all(q: Option<String>, state: &State<AppState>) -> ApiResult<Value> {
    let query = q.unwrap_or_default();
    if query.trim().is_empty() {
        return Ok(Json(json!({ "users": [], "projects": [], "competitions": [], "study_groups": [] })));
    }

    let users_raw = state.users.search(&query).await?;
    let users: Vec<UserResponse> = users_raw.into_iter().take(10).map(UserResponse::from).collect();

    let projects_raw = state.projects.search(&query).await?;
    let mut projects = Vec::new();
    for p in projects_raw.into_iter().take(10) {
        let lead_name = state.users.find_by_id(&p.lead_user_id).await?.map(|u| u.name).unwrap_or_default();
        projects.push(ProjectResponse::from_project(p, lead_name));
    }

    // Competitions: simple in-memory filter (no full-text index yet).
    let comps_raw = state.competitions.list().await?;
    let q_low = query.to_lowercase();
    let competitions: Vec<CompetitionResponse> = comps_raw
        .into_iter()
        .filter(|c| {
            c.name.to_lowercase().contains(&q_low)
                || c.description.to_lowercase().contains(&q_low)
                || c.tags.iter().any(|t| t.to_lowercase().contains(&q_low))
        })
        .take(10)
        .map(CompetitionResponse::from)
        .collect();

    let groups_raw: Vec<StudyGroup> = state.study_groups.list(100, 0).await?;
    let study_groups: Vec<_> = groups_raw
        .into_iter()
        .filter(|g| {
            g.name.to_lowercase().contains(&q_low)
                || g.subject.to_lowercase().contains(&q_low)
                || g.tags.iter().any(|t| t.to_lowercase().contains(&q_low))
        })
        .take(10)
        .collect();

    Ok(Json(json!({
        "users": users,
        "projects": projects,
        "competitions": competitions,
        "study_groups": study_groups,
    })))
}

pub fn routes() -> Vec<Route> {
    routes![search_all]
}
