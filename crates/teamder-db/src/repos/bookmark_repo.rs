use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::bookmark::Bookmark;

pub struct BookmarkRepo {
    collection: Collection<Bookmark>,
}

impl BookmarkRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Bookmark>("bookmarks"),
        }
    }

    pub async fn create(&self, bookmark: &Bookmark) -> Result<(), TeamderError> {
        self.collection
            .insert_one(bookmark)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove(
        &self,
        user_id: &str,
        kind: &str,
        entity_id: &str,
    ) -> Result<(), TeamderError> {
        self.collection
            .delete_one(doc! {
                "user_id": user_id,
                "kind": kind,
                "entity_id": entity_id,
            })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list(&self, user_id: &str) -> Result<Vec<Bookmark>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "user_id": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn exists(
        &self,
        user_id: &str,
        kind: &str,
        entity_id: &str,
    ) -> Result<bool, TeamderError> {
        let count = self
            .collection
            .count_documents(doc! {
                "user_id": user_id,
                "kind": kind,
                "entity_id": entity_id,
            })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(count > 0)
    }
}
