use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{
    error::TeamderError,
    models::bookmark::{Bookmark, BookmarkKind},
};

use crate::DbClient;

#[derive(Clone)]
pub struct BookmarkRepo {
    col: Collection<Bookmark>,
}

impl BookmarkRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("bookmarks") }
    }

    pub async fn create(&self, b: &Bookmark) -> Result<(), TeamderError> {
        self.col.insert_one(b).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<Bookmark>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! { "user_id": user_id }).with_options(opts)
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn exists(&self, user_id: &str, kind: &BookmarkKind, entity_id: &str) -> Result<bool, TeamderError> {
        let kind_bson = mongodb::bson::to_bson(kind).map_err(|e| TeamderError::Internal(e.to_string()))?;
        let n = self.col.count_documents(doc! {
            "user_id": user_id, "kind": kind_bson, "entity_id": entity_id,
        }).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(n > 0)
    }

    pub async fn delete(&self, user_id: &str, kind: &BookmarkKind, entity_id: &str) -> Result<(), TeamderError> {
        let kind_bson = mongodb::bson::to_bson(kind).map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col.delete_one(doc! {
            "user_id": user_id, "kind": kind_bson, "entity_id": entity_id,
        }).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
