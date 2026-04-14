use mongodb::{
    Collection,
    bson::{doc, Document},
};
use teamder_core::{
    error::TeamderError,
    models::user::{UpdateUserRequest, User},
};
use chrono::Utc;

use crate::DbClient;

pub struct UserRepo {
    col: Collection<User>,
}

impl UserRepo {
    pub fn new(db: &DbClient) -> Self {
        Self {
            col: db.db.collection("users"),
        }
    }

    pub async fn create(&self, user: &User) -> Result<(), TeamderError> {
        self.col
            .insert_one(user)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, TeamderError> {
        let filter = doc! { "_id": id };
        self.col
            .find_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, TeamderError> {
        let filter = doc! { "email": email };
        self.col
            .find_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(&self, limit: i64, skip: u64) -> Result<Vec<User>, TeamderError> {
        use mongodb::options::FindOptions;
        use futures_util::TryStreamExt;

        let opts = FindOptions::builder()
            .limit(limit)
            .skip(skip)
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self
            .col
            .find(doc! {})
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<User>, TeamderError> {
        use futures_util::TryStreamExt;
        let filter = doc! {
            "$or": [
                { "name": { "$regex": query, "$options": "i" } },
                { "role": { "$regex": query, "$options": "i" } },
                { "skill_tags": { "$regex": query, "$options": "i" } },
            ]
        };
        let cursor = self
            .col
            .find(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn update(
        &self,
        id: &str,
        req: &UpdateUserRequest,
    ) -> Result<(), TeamderError> {
        let mut update_doc = Document::new();

        if let Some(v) = &req.name {
            update_doc.insert("name", v.clone());
        }
        if let Some(v) = &req.role {
            update_doc.insert("role", v.clone());
        }
        if let Some(v) = &req.department {
            update_doc.insert("department", v.clone());
        }
        if let Some(v) = &req.year {
            update_doc.insert("year", v.clone());
        }
        if let Some(v) = &req.location {
            update_doc.insert("location", v.clone());
        }
        if let Some(v) = &req.bio {
            let bio_bson: Vec<_> = v.iter().map(|s| mongodb::bson::Bson::String(s.clone())).collect();
            update_doc.insert("bio", bio_bson);
        }
        if let Some(v) = &req.hours_per_week {
            update_doc.insert("hours_per_week", v.clone());
        }
        update_doc.insert("updated_at", Utc::now().to_rfc3339());

        let filter = doc! { "_id": id };
        let update = doc! { "$set": update_doc };

        self.col
            .update_one(filter, update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        let filter = doc! { "_id": id };
        self.col
            .delete_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.col
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
