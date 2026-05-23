use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::models::notification::Notification;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/notifications?<limit>")]
pub async fn list_notifications(
    state: &State<AppState>,
    auth: AuthUser,
    limit: Option<i64>,
) -> Result<Json<Vec<Notification>>, ApiError> {
    let limit = limit.unwrap_or(50);
    let notifs = state
        .db
        .notification_repo()
        .list(&auth.user_id, limit)
        .await?;
    Ok(Json(notifs))
}

#[rocket::post("/notifications/<id>/read")]
pub async fn mark_read(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let _auth = auth;
    state.db.notification_repo().mark_read(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/notifications/read-all")]
pub async fn read_all(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<SuccessResponse>, ApiError> {
    state
        .db
        .notification_repo()
        .mark_all_read(&auth.user_id)
        .await?;
    Ok(Json(SuccessResponse { success: true }))
}
