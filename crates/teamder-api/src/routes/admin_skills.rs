use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::skill_catalog::{
        CreateCategoryRequest, CreateTagRequest, StoredSkillCategory, StoredSkillTag,
        UpdateCategoryRequest, UpdateTagRequest,
    },
};

use crate::{error::ApiResult, guards::AdminUser, state::AppState};

/// GET /api/v1/admin/skills — full catalog including inactive tags.
#[get("/")]
async fn list_all(_admin: AdminUser, state: &State<AppState>) -> ApiResult<Value> {
    let cats = state.skill_catalog.list_categories().await?;
    let tags = state.skill_catalog.list_tags().await?;
    Ok(Json(json!({ "categories": cats, "tags": tags })))
}

// ── Categories ────────────────────────────────────────────────

#[post("/categories", data = "<req>")]
async fn create_category(
    req: Json<CreateCategoryRequest>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let key = req.0.key.trim().to_string();
    if key.is_empty() {
        return Err(TeamderError::Validation("Category key is required".into()).into());
    }
    if state.skill_catalog.category_exists(&key).await? {
        return Err(TeamderError::Conflict(format!("Category '{}' already exists", key)).into());
    }
    let cat = StoredSkillCategory::new(
        key,
        req.0.label.trim().to_string(),
        req.0.label_zh.trim().to_string(),
        req.0.order.unwrap_or(999),
    );
    state.skill_catalog.insert_category(&cat).await?;
    Ok(Json(json!({ "success": true, "key": cat.key })))
}

#[patch("/categories/<key>", data = "<req>")]
async fn update_category(
    key: String,
    req: Json<UpdateCategoryRequest>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    state.skill_catalog.update_category(&key, &req).await?;
    Ok(Json(json!({ "success": true })))
}

#[delete("/categories/<key>")]
async fn delete_category(
    key: String,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    state.skill_catalog.delete_category(&key).await?;
    Ok(Json(json!({ "success": true })))
}

// ── Tags ──────────────────────────────────────────────────────

#[post("/tags", data = "<req>")]
async fn create_tag(
    req: Json<CreateTagRequest>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let name = req.0.name.trim().to_string();
    if name.is_empty() {
        return Err(TeamderError::Validation("Tag name is required".into()).into());
    }
    let cat_key = req.0.category_key.trim().to_string();
    if !state.skill_catalog.category_exists(&cat_key).await? {
        return Err(TeamderError::Validation(format!("Unknown category '{}'", cat_key)).into());
    }
    let mut tag = StoredSkillTag::new(
        name,
        req.0.name_zh.trim().to_string(),
        cat_key,
        req.0.order.unwrap_or(999),
    );
    if let Some(active) = req.0.is_active {
        tag.is_active = active;
    }
    state.skill_catalog.insert_tag(&tag).await?;
    Ok(Json(json!({ "success": true, "id": tag.id })))
}

#[patch("/tags/<id>", data = "<req>")]
async fn update_tag(
    id: String,
    req: Json<UpdateTagRequest>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    state.skill_catalog.update_tag(&id, &req).await?;
    Ok(Json(json!({ "success": true })))
}

#[delete("/tags/<id>")]
async fn delete_tag(
    id: String,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    state.skill_catalog.delete_tag(&id).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![
        list_all,
        create_category, update_category, delete_category,
        create_tag, update_tag, delete_tag,
    ]
}
