use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{
    error::TeamderError,
    models::project_update::ProjectUpdate,
};

use crate::DbClient;

#[derive(Clone)]
pub struct ProjectUpdateRepo {
    col: Collection<ProjectUpdate>,
}

impl ProjectUpdateRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("project_updates") }
    }

    pub async fn create(&self, u: &ProjectUpdate) -> Result<(), TeamderError> {
        self.col.insert_one(u).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_project(&self, project_id: &str) -> Result<Vec<ProjectUpdate>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).limit(50).build();
        let cursor = self.col.find(doc! { "project_id": project_id }).with_options(opts)
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Recent updates across all projects, used for the home feed.
    pub async fn recent(&self, limit: i64) -> Result<Vec<ProjectUpdate>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).limit(limit).build();
        let cursor = self.col.find(doc! {}).with_options(opts)
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        self.col.delete_one(doc! { "_id": id })
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
