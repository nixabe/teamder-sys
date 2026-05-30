use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{error::TeamderError, models::study_group_announcement::StudyGroupAnnouncement};
use crate::DbClient;

pub struct StudyGroupAnnouncementRepo {
    col: Collection<StudyGroupAnnouncement>,
}

impl StudyGroupAnnouncementRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("study_group_announcements") }
    }

    pub async fn create(&self, ann: &StudyGroupAnnouncement) -> Result<(), TeamderError> {
        self.col.insert_one(ann).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_group(&self, group_id: &str) -> Result<Vec<StudyGroupAnnouncement>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "pinned": -1, "created_at": -1 })
            .build();
        let cursor = self.col
            .find(doc! { "group_id": group_id })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<StudyGroupAnnouncement>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        self.col.delete_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_for_group(&self, group_id: &str) -> Result<(), TeamderError> {
        self.col.delete_many(doc! { "group_id": group_id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
