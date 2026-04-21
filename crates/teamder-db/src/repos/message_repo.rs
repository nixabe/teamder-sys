use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{error::TeamderError, models::message::{ConversationSummary, Message}};
use crate::DbClient;

#[derive(Clone)]
pub struct MessageRepo {
    col: Collection<Message>,
}

impl MessageRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("messages") }
    }

    pub async fn create(&self, msg: &Message) -> Result<(), TeamderError> {
        self.col.insert_one(msg).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_conversation(
        &self,
        user_a: &str,
        user_b: &str,
        limit: i64,
        skip: u64,
    ) -> Result<Vec<Message>, TeamderError> {
        let filter = doc! {
            "$or": [
                { "from_user_id": user_a, "to_user_id": user_b },
                { "from_user_id": user_b, "to_user_id": user_a },
            ]
        };
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .skip(skip)
            .build();
        let cursor = self.col.find(filter).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        let mut msgs: Vec<Message> = cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        msgs.reverse();
        Ok(msgs)
    }

    /// Returns conversation summaries with partner_name = "" (caller enriches with user lookup).
    pub async fn list_conversations(&self, user_id: &str) -> Result<Vec<ConversationSummary>, TeamderError> {
        let filter = doc! {
            "$or": [{ "from_user_id": user_id }, { "to_user_id": user_id }]
        };
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();
        let cursor = self.col.find(filter).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        let msgs: Vec<Message> = cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        let mut seen = std::collections::HashSet::<String>::new();
        let mut result: Vec<ConversationSummary> = Vec::new();

        for msg in &msgs {
            let partner_id = if msg.from_user_id == user_id {
                msg.to_user_id.clone()
            } else {
                msg.from_user_id.clone()
            };
            let is_unread = msg.to_user_id == user_id && !msg.read;

            if !seen.contains(&partner_id) {
                seen.insert(partner_id.clone());
                result.push(ConversationSummary {
                    partner_id,
                    partner_name: String::new(), // enriched by route handler
                    last_message: msg.content.clone(),
                    last_at: msg.created_at,
                    unread_count: if is_unread { 1 } else { 0 },
                });
            } else if is_unread {
                if let Some(entry) = result.iter_mut().find(|c| c.partner_id == partner_id) {
                    entry.unread_count += 1;
                }
            }
        }
        Ok(result)
    }

    pub async fn mark_read(&self, from_user_id: &str, to_user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_many(
                doc! { "from_user_id": from_user_id, "to_user_id": to_user_id, "read": false },
                doc! { "$set": { "read": true } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
