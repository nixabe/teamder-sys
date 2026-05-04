use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    models::{
        competition::CompetitionResponse,
        project::ProjectResponse,
        study_group::StudyGroup,
        user::UserResponse,
    },
    skills::search_en_by_zh,
};

use crate::{error::ApiResult, state::AppState};

/// GET /api/v1/search?q=…
///
/// Cross-entity search: returns up to 10 hits each from users, projects,
/// competitions, and study groups. Supports both English and Traditional
/// Chinese skill names — a Chinese query is expanded into the matching
/// English skill names before the underlying repos run their searches.
#[get("/?<q>")]
async fn search_all(q: Option<String>, state: &State<AppState>) -> ApiResult<Value> {
    let query = q.unwrap_or_default();
    let raw_query = query.trim().to_string();
    if raw_query.is_empty() {
        return Ok(Json(json!({ "users": [], "projects": [], "competitions": [], "study_groups": [] })));
    }

    // Build expanded keyword list. Always includes the raw query; if the
    // query contains CJK characters or matches Chinese skill labels, also
    // include the corresponding English skill names so the English-stored
    // tags are searchable from a Chinese query.
    let mut keywords: Vec<String> = vec![raw_query.clone()];
    let zh_hits = search_en_by_zh(&raw_query);
    for k in zh_hits.iter() {
        keywords.push((*k).to_string());
    }
    keywords.sort();
    keywords.dedup();

    // Run user/project searches against every keyword and merge.
    let mut users_raw = Vec::new();
    let mut projects_raw = Vec::new();
    for kw in &keywords {
        users_raw.extend(state.users.search(kw).await?);
        projects_raw.extend(state.projects.search(kw).await?);
    }
    // Dedupe by id.
    users_raw.sort_by(|a, b| a.id.cmp(&b.id));
    users_raw.dedup_by(|a, b| a.id == b.id);
    projects_raw.sort_by(|a, b| a.id.cmp(&b.id));
    projects_raw.dedup_by(|a, b| a.id == b.id);

    let users: Vec<UserResponse> = users_raw.into_iter().take(10).map(UserResponse::from).collect();
    let mut projects = Vec::new();
    for p in projects_raw.into_iter().take(10) {
        let lead_name = state.users.find_by_id(&p.lead_user_id).await?.map(|u| u.name).unwrap_or_default();
        projects.push(ProjectResponse::from_project(p, lead_name));
    }

    // Competitions and study groups: in-memory contains-match against any keyword.
    let lower_keywords: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();
    let matches_any = |hay: &str| {
        let h = hay.to_lowercase();
        lower_keywords.iter().any(|k| !k.is_empty() && h.contains(k))
    };

    let comps_raw = state.competitions.list().await?;
    let competitions: Vec<CompetitionResponse> = comps_raw
        .into_iter()
        .filter(|c| {
            matches_any(&c.name)
                || matches_any(&c.description)
                || c.tags.iter().any(|t| matches_any(t))
        })
        .take(10)
        .map(CompetitionResponse::from)
        .collect();

    let groups_raw: Vec<StudyGroup> = state.study_groups.list(100, 0).await?;
    let study_groups: Vec<_> = groups_raw
        .into_iter()
        .filter(|g| {
            matches_any(&g.name)
                || matches_any(&g.subject)
                || g.tags.iter().any(|t| matches_any(t))
        })
        .take(10)
        .collect();

    Ok(Json(json!({
        "users": users,
        "projects": projects,
        "competitions": competitions,
        "study_groups": study_groups,
        "expanded_keywords": keywords,
    })))
}

pub fn routes() -> Vec<Route> {
    routes![search_all]
}
