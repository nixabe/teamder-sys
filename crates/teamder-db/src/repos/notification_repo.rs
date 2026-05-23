use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::notification::Notification;

pub struct NotificationRepo {
    collection: Collection<Notification>,
}

impl NotificationRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Notification>("notifications"),
        }
    }

    pub async fn create(&self, notif: &Notification) -> Result<(), TeamderError> {
        self.collection
            .insert_one(notif)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<Notification>, TeamderError> {
        let opts = FindOptions::builder()
            .limit(limit)
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self
            .collection
            .find(doc! { "user_id": user_id })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn mark_read(&self, id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": { "read": true } })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn mark_all_read(&self, user_id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_many(
                doc! { "user_id": user_id, "read": false },
                doc! { "$set": { "read": true } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count_unread(&self, user_id: &str) -> Result<u64, TeamderError> {
        self.collection
            .count_documents(doc! { "user_id": user_id, "read": false })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
