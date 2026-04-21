use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc};
use teamder_core::{error::TeamderError, models::join_request::{JoinRequest, JoinRequestStatus}};
use crate::DbClient;

pub struct JoinRequestRepo {
    col: Collection<JoinRequest>,
}

impl JoinRequestRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("join_requests") }
    }

    pub async fn create(&self, req: &JoinRequest) -> Result<(), TeamderError> {
        self.col.insert_one(req).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<JoinRequest>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Returns true if user already has a pending or accepted request for this entity.
    pub async fn exists_for_user(&self, user_id: &str, entity_id: &str) -> Result<bool, TeamderError> {
        let count = self.col
            .count_documents(doc! {
                "from_user_id": user_id,
                "entity_id": entity_id,
                "status": { "$in": ["pending", "accepted"] }
            })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(count > 0)
    }

    /// Requests sent by a user.
    pub async fn list_by_user(&self, user_id: &str) -> Result<Vec<JoinRequest>, TeamderError> {
        let cursor = self.col.find(doc! { "from_user_id": user_id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Pending requests for a given entity (for the owner to review).
    pub async fn list_pending_for_entity(&self, entity_id: &str) -> Result<Vec<JoinRequest>, TeamderError> {
        let cursor = self.col
            .find(doc! { "entity_id": entity_id, "status": "pending" })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// All pending requests owned by a user (across all their entities).
    pub async fn list_pending_for_owner(&self, owner_id: &str) -> Result<Vec<JoinRequest>, TeamderError> {
        let cursor = self.col
            .find(doc! { "owner_id": owner_id, "status": "pending" })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn update_status(&self, id: &str, status: &JoinRequestStatus) -> Result<(), TeamderError> {
        let status_bson = mongodb::bson::to_bson(status)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col
            .update_one(doc! { "_id": id }, doc! { "$set": { "status": status_bson } })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
