use chrono::Utc;
use mongodb::bson;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AdminUser;
use crate::state::AppState;
use teamder_core::models::skill_catalog::{StoredSkillCategory, StoredSkillTag};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdminSkillCatalog {
    pub categories: Vec<StoredSkillCategory>,
    pub tags: Vec<StoredSkillTag>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryBody {
    pub key: String,
    pub label: String,
    pub label_zh: String,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryBody {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub label_zh: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagBody {
    pub name: String,
    pub name_zh: String,
    pub category_key: String,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub name_zh: Option<String>,
    #[serde(default)]
    pub category_key: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/admin/skills")]
pub async fn list_skills(
    state: &State<AppState>,
    _admin: AdminUser,
) -> Result<Json<AdminSkillCatalog>, ApiError> {
    let categories = state.db.skill_catalog_repo().list_categories().await?;
    let tags = state.db.skill_catalog_repo().list_tags().await?;

    Ok(Json(AdminSkillCatalog { categories, tags }))
}

#[rocket::post("/admin/skills/categories", data = "<body>")]
pub async fn create_category(
    state: &State<AppState>,
    _admin: AdminUser,
    body: Json<CreateCategoryBody>,
) -> Result<Json<StoredSkillCategory>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();

    let cat = StoredSkillCategory {
        id: req.key,
        label: req.label,
        label_zh: req.label_zh,
        order: req.order.unwrap_or(0),
        created_at: now,
        updated_at: now,
    };

    state.db.skill_catalog_repo().create_category(&cat).await?;

    Ok(Json(cat))
}

#[rocket::patch("/admin/skills/categories/<key>", data = "<body>")]
pub async fn update_category(
    state: &State<AppState>,
    _admin: AdminUser,
    key: &str,
    body: Json<UpdateCategoryBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let req = body.into_inner();
    let mut update = bson::doc! {};

    if let Some(v) = &req.label {
        update.insert("label", v.as_str());
    }
    if let Some(v) = &req.label_zh {
        update.insert("label_zh", v.as_str());
    }
    if let Some(v) = req.order {
        update.insert("order", v);
    }

    update.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));

    state
        .db
        .skill_catalog_repo()
        .update_category(key, update)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::delete("/admin/skills/categories/<key>")]
pub async fn delete_category(
    state: &State<AppState>,
    _admin: AdminUser,
    key: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    state
        .db
        .skill_catalog_repo()
        .delete_category(key)
        .await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/admin/skills/tags", data = "<body>")]
pub async fn create_tag(
    state: &State<AppState>,
    _admin: AdminUser,
    body: Json<CreateTagBody>,
) -> Result<Json<StoredSkillTag>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();

    let tag = StoredSkillTag {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        name_zh: req.name_zh,
        category_key: req.category_key,
        order: req.order.unwrap_or(0),
        is_active: req.is_active.unwrap_or(true),
        created_at: now,
        updated_at: now,
    };

    state.db.skill_catalog_repo().create_tag(&tag).await?;

    Ok(Json(tag))
}

#[rocket::patch("/admin/skills/tags/<id>", data = "<body>")]
pub async fn update_tag(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
    body: Json<UpdateTagBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let req = body.into_inner();
    let mut update = bson::doc! {};

    if let Some(v) = &req.name {
        update.insert("name", v.as_str());
    }
    if let Some(v) = &req.name_zh {
        update.insert("name_zh", v.as_str());
    }
    if let Some(v) = &req.category_key {
        update.insert("category_key", v.as_str());
    }
    if let Some(v) = req.order {
        update.insert("order", v);
    }
    if let Some(v) = req.is_active {
        update.insert("is_active", v);
    }

    update.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));

    state
        .db
        .skill_catalog_repo()
        .update_tag(id, update)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::delete("/admin/skills/tags/<id>")]
pub async fn delete_tag(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    state.db.skill_catalog_repo().delete_tag(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}
