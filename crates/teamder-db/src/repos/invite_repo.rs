use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc};
use teamder_core::{error::TeamderError, models::invite::{Invite, InviteStatus}};
use crate::DbClient;

pub struct InviteRepo {
    col: Collection<Invite>,
}

impl InviteRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("invites") }
    }

    pub async fn create(&self, invite: &Invite) -> Result<(), TeamderError> {
        self.col.insert_one(invite).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Invite>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<Invite>, TeamderError> {
        let cursor = self.col
            .find(doc! {
                "$or": [
                    { "to_user_id": user_id },
                    { "from_user_id": user_id }
                ]
            })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn delete_by_id(&self, id: &str) -> Result<(), TeamderError> {
        self.col
            .delete_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_status(&self, id: &str, status: &InviteStatus) -> Result<(), TeamderError> {
        let status_bson = mongodb::bson::to_bson(status)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "status": status_bson } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
