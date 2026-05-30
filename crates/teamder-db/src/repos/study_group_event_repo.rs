use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{error::TeamderError, models::study_group_event::StudyGroupEvent};
use crate::DbClient;

pub struct StudyGroupEventRepo {
    col: Collection<StudyGroupEvent>,
}

impl StudyGroupEventRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("study_group_events") }
    }

    pub async fn create(&self, event: &StudyGroupEvent) -> Result<(), TeamderError> {
        self.col.insert_one(event).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_group(&self, group_id: &str) -> Result<Vec<StudyGroupEvent>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "starts_at": 1 })
            .build();
        let cursor = self.col
            .find(doc! { "group_id": group_id })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<StudyGroupEvent>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn add_attendee(&self, event_id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_one(doc! { "_id": event_id }, doc! { "$addToSet": { "attendees": user_id } })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_attendee(&self, event_id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_one(doc! { "_id": event_id }, doc! { "$pull": { "attendees": user_id } })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
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
