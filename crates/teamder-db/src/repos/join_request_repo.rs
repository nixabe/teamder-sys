use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::join_request::JoinRequest;

pub struct JoinRequestRepo {
    collection: Collection<JoinRequest>,
}

impl JoinRequestRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<JoinRequest>("join_requests"),
        }
    }

    pub async fn create(&self, request: &JoinRequest) -> Result<(), TeamderError> {
        self.collection
            .insert_one(request)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<JoinRequest>, TeamderError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Incoming requests for an entity owner (pending only).
    pub async fn incoming(&self, owner_id: &str) -> Result<Vec<JoinRequest>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "owner_id": owner_id, "status": "pending" })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Requests sent by a user.
    pub async fn sent(&self, user_id: &str) -> Result<Vec<JoinRequest>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "from_user_id": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn update_status(&self, id: &str, status: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": { "status": status } })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Find a pending request from a specific user for a specific entity.
    pub async fn find_pending_for_entity(
        &self,
        user_id: &str,
        entity_id: &str,
    ) -> Result<Option<JoinRequest>, TeamderError> {
        self.collection
            .find_one(doc! {
                "from_user_id": user_id,
                "entity_id": entity_id,
                "status": "pending",
            })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
