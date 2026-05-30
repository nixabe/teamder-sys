use rocket::{Route, State, serde::json::Json};
use rocket::futures::{SinkExt, StreamExt};
use rocket_ws as ws;
use serde_json::{Value, json};
use teamder_core::models::notification::NotificationResponse;

use crate::{auth::verify_token, error::{ApiResult, ApiError}, state::AppState, guards::AuthUser};

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

/// GET /api/v1/notifications/ws?<token>  — real-time push channel
#[get("/ws?<token>")]
pub async fn notif_ws(
    ws: ws::WebSocket,
    token: String,
    state: &State<AppState>,
) -> Result<ws::Channel<'static>, ApiError> {
    let claims = verify_token(&token, &state.jwt_secret)?;
    let user_id = claims.sub.clone();
    let notif_hub = state.notif_hub.clone();
    let mut rx = notif_hub.subscribe(&user_id).await;

    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            loop {
                tokio::select! {
                    frame = stream.next() => {
                        // client closed
                        if frame.is_none() { break; }
                    }
                    result = rx.recv() => {
                        match result {
                            Ok(text) => {
                                let _ = stream.send(ws::Message::Text(text.into())).await;
                            }
                            _ => break,
                        }
                    }
                }
            }
            Ok(())
        })
    }))
}

pub fn routes() -> Vec<Route> {
    routes![list_mine, mark_read, mark_all_read, notif_ws]
}
