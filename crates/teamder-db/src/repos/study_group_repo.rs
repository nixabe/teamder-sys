use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc};
use teamder_core::{error::TeamderError, models::study_group::StudyGroup};
use crate::DbClient;

pub struct StudyGroupRepo {
    col: Collection<StudyGroup>,
}

impl StudyGroupRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("study_groups") }
    }

    pub async fn create(&self, group: &StudyGroup) -> Result<(), TeamderError> {
        self.col.insert_one(group).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<StudyGroup>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(&self, limit: i64, skip: u64) -> Result<Vec<StudyGroup>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder()
            .limit(limit)
            .skip(skip)
            .sort(doc! { "created_at": -1 })
            .build();
        let cursor = self.col.find(doc! {}).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_by_member(&self, user_id: &str) -> Result<Vec<StudyGroup>, TeamderError> {
        let cursor = self.col.find(doc! { "members.user_id": user_id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_by_creator(&self, user_id: &str) -> Result<Vec<StudyGroup>, TeamderError> {
        let cursor = self.col.find(doc! { "created_by": user_id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_open(&self) -> Result<Vec<StudyGroup>, TeamderError> {
        let cursor = self.col.find(doc! { "is_open": true }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn add_member(
        &self,
        group_id: &str,
        member: &teamder_core::models::study_group::GroupMember,
    ) -> Result<(), TeamderError> {
        let member_bson = mongodb::bson::to_bson(member)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col
            .update_one(
                doc! { "_id": group_id },
                doc! { "$push": { "members": member_bson } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn checkin(&self, group_id: &str, user_id: &str) -> Result<(), TeamderError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.col
            .update_one(
                doc! { "_id": group_id, "members.user_id": user_id },
                doc! {
                    "$set": {
                        "members.$.last_checkin": now,
                        "updated_at": chrono::Utc::now().to_rfc3339(),
                    },
                    "$inc": { "members.$.streak": 1 }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn add_note(
        &self,
        group_id: &str,
        note: &teamder_core::models::study_group::StudyNote,
    ) -> Result<(), TeamderError> {
        let note_bson = mongodb::bson::to_bson(note)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col
            .update_one(
                doc! { "_id": group_id },
                doc! { "$push": { "notes": note_bson } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_note(
        &self,
        group_id: &str,
        note_id: &str,
    ) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": group_id },
                doc! { "$pull": { "notes": { "id": note_id } } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_member(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": group_id },
                doc! { "$pull": { "members": { "user_id": user_id } } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn set_status(
        &self,
        group_id: &str,
        status: &str,
    ) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": group_id },
                doc! { "$set": { "status": status, "updated_at": chrono::Utc::now().to_rfc3339() } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_progress(
        &self,
        group_id: &str,
        current_week: u8,
    ) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": group_id },
                doc! { "$set": {
                    "current_week": current_week as i32,
                    "updated_at": chrono::Utc::now().to_rfc3339(),
                } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.col.count_documents(doc! {}).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
