use std::collections::HashMap;
use rocket::{State, serde::json::Json};
use rocket::futures::{SinkExt, StreamExt};
use rocket_ws as ws;
use teamder_core::models::message::{ConversationSummary, Message, MessageResponse, WsIncoming};

use crate::{
    auth::verify_token,
    error::{ApiError, ApiResult},
    guards::AuthUser,
    state::AppState,
};

fn build_msg_response(msg: &Message, from_user_name: &str) -> MessageResponse {
    MessageResponse {
        id: msg.id.clone(),
        from_user_id: msg.from_user_id.clone(),
        from_user_name: from_user_name.to_string(),
        to_user_id: msg.to_user_id.clone(),
        content: msg.content.clone(),
        created_at: msg.created_at,
        read: msg.read,
    }
}

#[get("/conversations")]
pub async fn list_conversations(
    user: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Vec<ConversationSummary>> {
    let mut convs = state.messages.list_conversations(&user.0.sub).await?;

    if !convs.is_empty() {
        let partner_ids: Vec<&str> = convs.iter().map(|c| c.partner_id.as_str()).collect();
        let users = state.users.find_many_by_ids(&partner_ids).await?;
        let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();
        for conv in &mut convs {
            conv.partner_name = names.get(conv.partner_id.as_str()).copied().unwrap_or("").to_string();
        }
    }

    Ok(Json(convs))
}

#[get("/messages/<partner_id>?<limit>&<skip>")]
pub async fn message_history(
    user: AuthUser,
    partner_id: String,
    limit: Option<i64>,
    skip: Option<u64>,
    state: &State<AppState>,
) -> ApiResult<Vec<MessageResponse>> {
    let msgs = state
        .messages
        .list_conversation(
            &user.0.sub,
            &partner_id,
            limit.unwrap_or(50),
            skip.unwrap_or(0),
        )
        .await?;
    let _ = state.messages.mark_read(&partner_id, &user.0.sub).await;

    // Batch-fetch the two participants' names
    let user_ids: Vec<&str> = [user.0.sub.as_str(), partner_id.as_str()].into();
    let users = state.users.find_many_by_ids(&user_ids).await?;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();

    let responses = msgs.iter()
        .map(|m| {
            let name = names.get(m.from_user_id.as_str()).copied().unwrap_or("");
            build_msg_response(m, name)
        })
        .collect();

    Ok(Json(responses))
}

#[get("/ws?<token>")]
pub async fn chat_ws(
    ws: ws::WebSocket,
    token: String,
    state: &State<AppState>,
) -> Result<ws::Channel<'static>, ApiError> {
    let claims = verify_token(&token, &state.jwt_secret)?;
    let user_id = claims.sub.clone();

    // Look up the connecting user's name once at connect time
    let user_name = state.users.find_by_id(&user_id).await
        .ok().flatten()
        .map(|u| u.name)
        .unwrap_or_default();

    let msg_repo = state.messages.clone();
    let chat = state.chat.clone();

    let mut rx = chat.subscribe(&user_id).await;

    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            loop {
                tokio::select! {
                    frame = stream.next() => {
                        let Some(Ok(ws::Message::Text(text))) = frame else { break };
                        let Ok(incoming) = serde_json::from_str::<WsIncoming>(&text.to_string()) else { continue };
                        if incoming.content.trim().is_empty() { continue; }

                        let m = Message::new(&user_id, &incoming.to_user_id, &incoming.content);
                        let resp = build_msg_response(&m, &user_name);
                        let _ = msg_repo.create(&m).await;

                        if let Ok(json) = serde_json::to_string(&resp) {
                            chat.send_to(&incoming.to_user_id, json.clone()).await;
                            chat.send_to(&user_id, json).await;
                        }
                    }
                    result = rx.recv() => {
                        match result {
                            Ok(text) => { let _ = stream.send(ws::Message::Text(text.into())).await; }
                            _ => break,
                        }
                    }
                }
            }
            Ok(())
        })
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![list_conversations, message_history, chat_ws]
}
