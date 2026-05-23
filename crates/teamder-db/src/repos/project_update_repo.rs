use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::project_update::ProjectUpdate;

pub struct ProjectUpdateRepo {
    collection: Collection<ProjectUpdate>,
}

impl ProjectUpdateRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<ProjectUpdate>("project_updates"),
        }
    }

    pub async fn create(&self, update: &ProjectUpdate) -> Result<(), TeamderError> {
        self.collection
            .insert_one(update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_by_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectUpdate>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();
        let cursor = self
            .collection
            .find(doc! { "project_id": project_id })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        self.collection
            .delete_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
