use rocket::futures::{SinkExt, StreamExt};
use rocket::{State, serde::json::Json};
use rocket_ws as ws;
use teamder_core::models::message::{ConversationSummary, Message, MessageResponse, WsIncoming};

use crate::{
    auth::verify_token,
    error::{ApiError, ApiResult},
    guards::AuthUser,
    state::AppState,
};

#[get("/conversations")]
pub async fn list_conversations(
    user: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Vec<ConversationSummary>> {
    let conversations = state.messages.list_conversations(&user.0.sub).await?;
    Ok(Json(conversations))
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
    Ok(Json(msgs.into_iter().map(MessageResponse::from).collect()))
}

#[get("/ws?<token>")]
pub async fn chat_ws(
    ws: ws::WebSocket,
    token: String,
    state: &State<AppState>,
) -> Result<ws::Channel<'static>, ApiError> {
    let claims = verify_token(&token, &state.jwt_secret)?;
    let user_id = claims.sub.clone();

    let user_name = state
        .users
        .find_by_id(&user_id)
        .await
        .ok()
        .flatten()
        .map(|u| u.name)
        .unwrap_or_default();

    let msg_repo = state.messages.clone();
    let chat = state.chat.clone();
    let users = state.users.clone();

    let mut rx = chat.subscribe(&user_id).await;

    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            loop {
                tokio::select! {
                    frame = stream.next() => {
                        let Some(Ok(ws::Message::Text(text))) = frame else { break };
                        let text_str = text.to_string();
                        let Ok(incoming) = serde_json::from_str::<WsIncoming>(&text_str) else { continue };
                        if incoming.content.trim().is_empty() { continue; }

                        let to_name = users
                            .find_by_id(&incoming.to_user_id)
                            .await
                            .ok()
                            .flatten()
                            .map(|u| u.name)
                            .unwrap_or_default();

                        let m = Message::new(
                            &user_id,
                            &user_name,
                            &incoming.to_user_id,
                            &to_name,
                            &incoming.content,
                        );
                        let resp = MessageResponse::from(m.clone());
                        let _ = msg_repo.create(&m).await;

                        if let Ok(json) = serde_json::to_string(&resp) {
                            chat.send_to(&incoming.to_user_id, json.clone()).await;
                            // Echo to sender so their UI updates immediately
                            chat.send_to(&user_id, json).await;
                        }
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

pub fn routes() -> Vec<rocket::Route> {
    routes![list_conversations, message_history, chat_ws]
}
