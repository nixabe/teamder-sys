use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::skills::catalog as default_catalog;

use crate::{error::ApiResult, state::AppState};

/// GET /api/v1/skills — return the live (DB-backed) skill catalog grouped by
/// category. Falls back to the hardcoded default if the DB is empty (which
/// shouldn't normally happen because the seed runs at boot).
#[get("/")]
async fn get_catalog(state: &State<AppState>) -> ApiResult<Value> {
    let cats = state.skill_catalog.list_categories().await?;
    let tags = state.skill_catalog.list_active_tags().await?;

    if cats.is_empty() {
        // Fallback to hardcoded default.
        let cats = default_catalog();
        return Ok(Json(json!({ "categories": cats })));
    }

    let categories: Vec<Value> = cats
        .into_iter()
        .map(|c| {
            let skills: Vec<Value> = tags
                .iter()
                .filter(|t| t.category_key == c.key)
                .map(|t| json!({ "name": t.name, "name_zh": t.name_zh }))
                .collect();
            json!({
                "key": c.key,
                "label": c.label,
                "label_zh": c.label_zh,
                "skills": skills,
            })
        })
        .collect();

    Ok(Json(json!({ "categories": categories })))
}

pub fn routes() -> Vec<Route> {
    routes![get_catalog]
}
