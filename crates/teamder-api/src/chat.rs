use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

#[derive(Clone)]
pub struct ChatState {
    inner: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, user_id: &str) -> broadcast::Receiver<String> {
        let mut map = self.inner.write().await;
        map.entry(user_id.to_string())
            .or_insert_with(|| broadcast::channel::<String>(128).0)
            .subscribe()
    }

    pub async fn send_to(&self, user_id: &str, msg: String) {
        let map = self.inner.read().await;
        if let Some(tx) = map.get(user_id) {
            let _ = tx.send(msg);
        }
    }
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}
