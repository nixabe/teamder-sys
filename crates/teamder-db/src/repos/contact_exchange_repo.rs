use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOneOptions};
use teamder_core::{
    error::TeamderError,
    models::contact_exchange::{ContactExchange, ContactExchangeStatus},
};
use crate::DbClient;

pub struct ContactExchangeRepo {
    col: Collection<ContactExchange>,
}

impl ContactExchangeRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("contact_exchanges") }
    }

    pub async fn create(&self, exchange: &ContactExchange) -> Result<(), TeamderError> {
        self.col.insert_one(exchange).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<ContactExchange>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Find the most recent exchange between two users (any status).
    pub async fn find_between(
        &self,
        user_a: &str,
        user_b: &str,
    ) -> Result<Option<ContactExchange>, TeamderError> {
        let filter = doc! {
            "$or": [
                { "from_user_id": user_a, "to_user_id": user_b },
                { "from_user_id": user_b, "to_user_id": user_a },
            ]
        };
        let opts = FindOneOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();
        self.col.find_one(filter).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Find all exchanges involving a user (for cleanup).
    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<ContactExchange>, TeamderError> {
        let filter = doc! {
            "$or": [{ "from_user_id": user_id }, { "to_user_id": user_id }]
        };
        let cursor = self.col.find(filter).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: &ContactExchangeStatus,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), TeamderError> {
        let status_bson = mongodb::bson::to_bson(status)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        let mut set_doc = doc! { "status": status_bson };
        if let Some(exp) = expires_at {
            set_doc.insert("expires_at", mongodb::bson::DateTime::from_millis(exp.timestamp_millis()));
        }
        self.col
            .update_one(doc! { "_id": id }, doc! { "$set": set_doc })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_for_user(&self, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .delete_many(doc! {
                "$or": [{ "from_user_id": user_id }, { "to_user_id": user_id }]
            })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
