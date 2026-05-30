use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{
    error::TeamderError,
    models::report::{Report, ReportStatus},
};

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

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Report>, TeamderError> {
        self.col
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Apply an admin review to a report. `$set`s only the provided fields; when a
    /// `status` is given it also stamps `reviewed_by`/`reviewed_at`.
    pub async fn update_review(
        &self,
        id: &str,
        status: Option<ReportStatus>,
        reviewer_id: &str,
        admin_notes: Option<String>,
    ) -> Result<(), TeamderError> {
        use mongodb::bson::{to_bson, Document};

        let mut set_doc = Document::new();
        if let Some(s) = status {
            let bson = to_bson(&s).map_err(|e| TeamderError::Database(e.to_string()))?;
            set_doc.insert("status", bson);
            set_doc.insert("reviewed_by", reviewer_id.to_string());
            set_doc.insert("reviewed_at", Utc::now().to_rfc3339());
        }
        if let Some(notes) = admin_notes {
            set_doc.insert("admin_notes", notes);
        }
        if set_doc.is_empty() {
            return Ok(());
        }
        self.col
            .update_one(doc! { "_id": id }, doc! { "$set": set_doc })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
