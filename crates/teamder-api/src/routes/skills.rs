use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

use crate::error::ApiError;
use crate::state::AppState;
use teamder_core::models::skill_catalog::{StoredSkillCategory, StoredSkillTag};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SkillCatalogResponse {
    pub categories: Vec<StoredSkillCategory>,
    pub tags: Vec<StoredSkillTag>,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/skills/catalog")]
pub async fn get_catalog(
    state: &State<AppState>,
) -> Result<Json<SkillCatalogResponse>, ApiError> {
    let categories = state.db.skill_catalog_repo().list_categories().await?;
    let tags = state.db.skill_catalog_repo().list_tags().await?;

    Ok(Json(SkillCatalogResponse { categories, tags }))
}
