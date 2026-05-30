use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{error::TeamderError, models::report::Report};

use crate::DbClient;

#[derive(Clone)]
pub struct ReportRepo {
    col: Collection<Report>,
}

impl ReportRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("reports") }
    }

    pub async fn create(&self, r: &Report) -> Result<(), TeamderError> {
        self.col.insert_one(r).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<Report>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! {}).with_options(opts)
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }
}
