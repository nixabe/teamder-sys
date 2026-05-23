use chrono::Utc;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::verify_token;
use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::message::Message;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SendMessageBody {
    pub to_user_id: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ConversationWithPartner {
    pub partner_id: String,
    pub partner_name: String,
    pub partner_avatar: Option<String>,
    pub partner_initials: String,
    pub partner_gradient: String,
    pub last_message: String,
    pub unread_count: i64,
    pub updated_at: chrono::DateTime<Utc>,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/chat/conversations")]
pub async fn list_conversations(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ConversationWithPartner>>, ApiError> {
    let summaries = state
        .db
        .message_repo()
        .conversations(&auth.user_id)
        .await?;

    let mut convos = Vec::new();
    for s in summaries {
        let partner = state.db.user_repo().find_by_id(&s.partner_id).await?;
        let (name, avatar, initials, gradient) = match partner {
            Some(u) => (
                u.name,
                u.avatar_url,
                u.initials,
                u.gradient,
            ),
            None => (
                "Unknown".to_string(),
                None,
                "??".to_string(),
                String::new(),
            ),
        };

        convos.push(ConversationWithPartner {
            partner_id: s.partner_id,
            partner_name: name,
            partner_avatar: avatar,
            partner_initials: initials,
            partner_gradient: gradient,
            last_message: s.last_message,
            unread_count: s.unread_count,
            updated_at: s.updated_at,
        });
    }

    Ok(Json(convos))
}

#[rocket::get("/chat/messages/<partner_id>")]
pub async fn get_messages(
    state: &State<AppState>,
    auth: AuthUser,
    partner_id: &str,
) -> Result<Json<Vec<Message>>, ApiError> {
    let messages = state
        .db
        .message_repo()
        .messages_with(&auth.user_id, partner_id)
        .await?;

    // Mark messages from partner as read
    let _ = state
        .db
        .message_repo()
        .mark_read(partner_id, &auth.user_id)
        .await;

    Ok(Json(messages))
}

#[rocket::post("/chat/messages", data = "<body>")]
pub async fn send_message(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<SendMessageBody>,
) -> Result<Json<Message>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();

    let msg = Message {
        id: Uuid::new_v4().to_string(),
        from_user_id: auth.user_id.clone(),
        to_user_id: req.to_user_id.clone(),
        content: req.content.clone(),
        read: false,
        created_at: now,
    };

    state.db.message_repo().create(&msg).await?;

    // Broadcast to recipient's WebSocket channel
    let tx = state
        .chat_state
        .get_or_create_channel(&req.to_user_id);
    let payload = serde_json::to_string(&msg).unwrap_or_default();
    let _ = tx.send(payload);

    Ok(Json(msg))
}

// ── WebSocket handler ───────────────────────────────────────────────────────

#[rocket::get("/chat/ws?<token>")]
pub fn websocket_handler(
    state: &State<AppState>,
    ws: rocket_ws::WebSocket,
    token: String,
) -> Result<rocket_ws::Channel<'static>, ApiError> {
    let claims = verify_token(&token, &state.jwt_secret)
        .map_err(|_| TeamderError::Unauthorized("Invalid token".into()))?;

    let user_id = claims.sub;
    let chat_state = state.chat_state.channels.clone();
    let db = state.db.clone();

    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            use rocket_ws::Message as WsMsg;
            use tokio::sync::broadcast;

            // Subscribe to this user's channel
            let tx = {
                let mut map = chat_state.lock().unwrap();
                map.entry(user_id.clone())
                    .or_insert_with(|| {
                        let (tx, _) = broadcast::channel(64);
                        tx
                    })
                    .clone()
            };
            let mut rx = tx.subscribe();

            loop {
                tokio::select! {
                    // Messages from WebSocket client
                    msg = futures::StreamExt::next(&mut stream) => {
                        match msg {
                            Some(Ok(WsMsg::Text(text))) => {
                                // Parse incoming message
                                #[derive(serde::Deserialize)]
                                struct WsIncoming {
                                    to_user_id: String,
                                    content: String,
                                }

                                if let Ok(incoming) = serde_json::from_str::<WsIncoming>(&text) {
                                    let now = chrono::Utc::now();
                                    let msg = Message {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        from_user_id: user_id.clone(),
                                        to_user_id: incoming.to_user_id.clone(),
                                        content: incoming.content,
                                        read: false,
                                        created_at: now,
                                    };

                                    let _ = db.message_repo().create(&msg).await;

                                    // Broadcast to recipient
                                    let recipient_tx = {
                                        let mut map = chat_state.lock().unwrap();
                                        map.entry(incoming.to_user_id)
                                            .or_insert_with(|| {
                                                let (tx, _) = broadcast::channel(64);
                                                tx
                                            })
                                            .clone()
                                    };
                                    let payload = serde_json::to_string(&msg).unwrap_or_default();
                                    let _ = recipient_tx.send(payload);
                                }
                            }
                            Some(Ok(WsMsg::Close(_))) | None => break,
                            _ => {}
                        }
                    }
                    // Messages from broadcast channel (incoming from other users)
                    result = rx.recv() => {
                        match result {
                            Ok(payload) => {
                                if futures::SinkExt::send(
                                    &mut stream,
                                    WsMsg::Text(payload.into()),
                                ).await.is_err() {
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(_) => break,
                        }
                    }
                }
            }

            Ok(())
        })
    }))
}
