use futures_util::TryStreamExt;
use mongodb::{
    Collection,
    bson::doc,
    options::FindOptions,
};
use teamder_core::{
    error::TeamderError,
    models::notification::Notification,
};

use crate::DbClient;

#[derive(Clone)]
pub struct NotificationRepo {
    col: Collection<Notification>,
}

impl NotificationRepo {
    pub fn new(db: &DbClient) -> Self {
        Self {
            col: db.db.collection("notifications"),
        }
    }

    pub async fn create(&self, n: &Notification) -> Result<(), TeamderError> {
        self.col
            .insert_one(n)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: &str, limit: i64) -> Result<Vec<Notification>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .build();
        let cursor = self
            .col
            .find(doc! { "user_id": user_id })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn unread_count(&self, user_id: &str) -> Result<u64, TeamderError> {
        self.col
            .count_documents(doc! { "user_id": user_id, "read": false })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn mark_read(&self, id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": id, "user_id": user_id },
                doc! { "$set": { "read": true } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn mark_all_read(&self, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_many(
                doc! { "user_id": user_id, "read": false },
                doc! { "$set": { "read": true } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
