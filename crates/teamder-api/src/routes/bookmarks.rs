use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::bookmark::{Bookmark, BookmarkKind, BookmarkResponse, CreateBookmarkRequest},
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// GET /api/v1/bookmarks
#[get("/")]
async fn list_mine(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let raw = state.bookmarks.list_for_user(&auth.0.sub).await?;
    let data: Vec<BookmarkResponse> = raw.into_iter().map(Into::into).collect();
    Ok(Json(json!({ "data": data })))
}

/// POST /api/v1/bookmarks
#[post("/", data = "<req>")]
async fn add(
    req: Json<CreateBookmarkRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    if state.bookmarks.exists(&auth.0.sub, &req.kind, &req.entity_id).await? {
        return Err(TeamderError::Conflict("Already bookmarked".into()).into());
    }
    let b = Bookmark::new(&auth.0.sub, req.0.kind, req.0.entity_id, req.0.label);
    state.bookmarks.create(&b).await?;
    Ok(Json(json!({ "id": b.id })))
}

#[derive(Debug, serde::Deserialize)]
struct RemoveBookmarkRequest {
    kind: BookmarkKind,
    entity_id: String,
}

/// POST /api/v1/bookmarks/remove  (idempotent)
#[post("/remove", data = "<req>")]
async fn remove(
    req: Json<RemoveBookmarkRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    state.bookmarks.delete(&auth.0.sub, &req.kind, &req.entity_id).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_mine, add, remove]
}
