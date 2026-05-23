use chrono::Utc;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::models::bookmark::Bookmark;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddBookmarkBody {
    pub kind: String,
    pub entity_id: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveBookmarkBody {
    pub kind: String,
    pub entity_id: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/bookmarks")]
pub async fn list_bookmarks(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Bookmark>>, ApiError> {
    let bookmarks = state
        .db
        .bookmark_repo()
        .list(&auth.user_id)
        .await?;
    Ok(Json(bookmarks))
}

#[rocket::post("/bookmarks", data = "<body>")]
pub async fn add_bookmark(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<AddBookmarkBody>,
) -> Result<Json<Bookmark>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();

    let bookmark = Bookmark {
        id: Uuid::new_v4().to_string(),
        user_id: auth.user_id,
        kind: req.kind,
        entity_id: req.entity_id,
        label: req.label.unwrap_or_default(),
        created_at: now,
    };

    state.db.bookmark_repo().create(&bookmark).await?;

    Ok(Json(bookmark))
}

#[rocket::post("/bookmarks/remove", data = "<body>")]
pub async fn remove_bookmark(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<RemoveBookmarkBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let req = body.into_inner();

    state
        .db
        .bookmark_repo()
        .remove(&auth.user_id, &req.kind, &req.entity_id)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}
