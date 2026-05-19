use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc};
use teamder_core::{error::TeamderError, models::competition::{Competition, PublishStatus}};
use crate::DbClient;

pub struct CompetitionRepo {
    col: Collection<Competition>,
}

impl CompetitionRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("competitions") }
    }

    pub async fn create(&self, comp: &Competition) -> Result<(), TeamderError> {
        self.col.insert_one(comp).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Competition>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(&self) -> Result<Vec<Competition>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! { "publish_status": "published" }).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_featured(&self) -> Result<Vec<Competition>, TeamderError> {
        let cursor = self.col.find(doc! { "is_featured": true, "publish_status": "published" }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_all(&self) -> Result<Vec<Competition>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! {}).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_by_publisher(&self, publisher_id: &str) -> Result<Vec<Competition>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! { "publisher_id": publisher_id }).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_pending(&self) -> Result<Vec<Competition>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! { "publish_status": "pending_review" }).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn set_publish_status(
        &self,
        id: &str,
        status: &PublishStatus,
        rejected_note: Option<&str>,
    ) -> Result<(), TeamderError> {
        let status_str = serde_json::to_value(status)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        let mut patch = doc! {
            "publish_status": &status_str,
            "updated_at": Utc::now().to_rfc3339(),
        };
        match rejected_note {
            Some(note) => { patch.insert("rejected_note", note); }
            None => { patch.insert("rejected_note", mongodb::bson::Bson::Null); }
        }
        self.col
            .update_one(doc! { "_id": id }, doc! { "$set": patch })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn add_registration(
        &self,
        comp_id: &str,
        reg: &teamder_core::models::competition::Registration,
    ) -> Result<(), TeamderError> {
        let reg_bson = mongodb::bson::to_bson(reg)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col
            .update_one(
                doc! { "_id": comp_id },
                doc! { "$push": { "registrations": reg_bson } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Add user to interested list (idempotent).
    pub async fn add_interested(&self, comp_id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": comp_id },
                doc! { "$addToSet": { "interested_user_ids": user_id } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_interested(&self, comp_id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": comp_id },
                doc! { "$pull": { "interested_user_ids": user_id } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn set_winners(&self, comp_id: &str, winners: Vec<String>) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": comp_id },
                doc! { "$set": { "winners": winners } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update(&self, id: &str, patch: mongodb::bson::Document) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": id },
                doc! { "$set": patch },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        self.col
            .delete_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.col.count_documents(doc! {}).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
