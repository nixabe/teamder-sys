use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

use crate::error::ApiError;
use crate::state::AppState;
use teamder_core::error::TeamderError;
// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub kind: String, // "user" | "project" | "competition" | "study_group"
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/search?<q>&<kind>&<limit>")]
pub async fn search(
    state: &State<AppState>,
    q: String,
    kind: Option<String>,
    limit: Option<i64>,
) -> Result<Json<SearchResponse>, ApiError> {
    if q.is_empty() {
        return Err(TeamderError::Validation("Query parameter q is required".into()).into());
    }

    let limit = limit.unwrap_or(20);
    let type_filter = kind.as_deref();

    let mut results = Vec::new();

    // Search users
    if type_filter.is_none() || type_filter == Some("user") {
        let users = state.db.user_repo().search(&q, limit).await?;
        for u in users {
            results.push(SearchResultItem {
                kind: "user".to_string(),
                id: u.id,
                name: u.name,
                description: u.headline,
                icon: u.avatar_url,
            });
        }
    }

    // Search projects
    if type_filter.is_none() || type_filter == Some("project") {
        let projects = state.db.project_repo().search(&q, limit).await?;
        for p in projects {
            results.push(SearchResultItem {
                kind: "project".to_string(),
                id: p.id,
                name: p.name,
                description: Some(p.description),
                icon: Some(p.icon),
            });
        }
    }

    // Search competitions
    if type_filter.is_none() || type_filter == Some("competition") {
        let comps = state.db.competition_repo().search(&q, limit).await?;
        for c in comps {
            results.push(SearchResultItem {
                kind: "competition".to_string(),
                id: c.id,
                name: c.name,
                description: Some(c.description),
                icon: Some(c.icon),
            });
        }
    }

    // Search study groups
    if type_filter.is_none() || type_filter == Some("study_group") {
        let groups = state.db.study_group_repo().search(&q, limit).await?;
        for g in groups {
            results.push(SearchResultItem {
                kind: "study_group".to_string(),
                id: g.id,
                name: g.name,
                description: g.description,
                icon: Some(g.icon),
            });
        }
    }

    // Limit total results
    results.truncate(limit as usize);

    Ok(Json(SearchResponse { results }))
}
