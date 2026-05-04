use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::models::notification::NotificationResponse;

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// GET /api/v1/notifications  — current user's notifications + unread count.
#[get("/")]
async fn list_mine(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let raw = state.notifications.list_for_user(&auth.0.sub, 100).await?;
    let unread = state.notifications.unread_count(&auth.0.sub).await?;
    let data: Vec<NotificationResponse> = raw.into_iter().map(Into::into).collect();
    Ok(Json(json!({ "data": data, "unread": unread })))
}

/// POST /api/v1/notifications/<id>/read
#[post("/<id>/read")]
async fn mark_read(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    state.notifications.mark_read(&id, &auth.0.sub).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/notifications/read-all
#[post("/read-all")]
async fn mark_all_read(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    state.notifications.mark_all_read(&auth.0.sub).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_mine, mark_read, mark_all_read]
}
