use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use teamder_db::client::DbClient;
use tokio::sync::broadcast;

/// Application-wide state managed by Rocket.
///
/// Injected into route handlers via `&State<AppState>`.
pub struct AppState {
    pub db: DbClient,
    pub jwt_secret: String,
    pub chat_state: ChatState,
}

/// Manages per-user broadcast channels for real-time chat via WebSocket.
pub struct ChatState {
    pub channels: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns an existing broadcast sender for the given user_id,
    /// or creates a new one with a capacity of 64 messages.
    pub fn get_or_create_channel(&self, user_id: &str) -> broadcast::Sender<String> {
        let mut map = self.channels.lock().unwrap();
        map.entry(user_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(64);
                tx
            })
            .clone()
    }
}
