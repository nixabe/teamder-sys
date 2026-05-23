use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{self, doc};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::study_group::{GroupMember, StudyGroup, StudyNote};

pub struct StudyGroupRepo {
    collection: Collection<StudyGroup>,
}

impl StudyGroupRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<StudyGroup>("study_groups"),
        }
    }

    pub async fn create(&self, group: &StudyGroup) -> Result<(), TeamderError> {
        self.collection
            .insert_one(group)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<StudyGroup>, TeamderError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(
        &self,
        open_only: bool,
        skip: u64,
        limit: i64,
    ) -> Result<(Vec<StudyGroup>, u64), TeamderError> {
        let filter = if open_only {
            doc! { "is_open": true }
        } else {
            doc! {}
        };

        let total = self
            .collection
            .count_documents(filter.clone())
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        let opts = FindOptions::builder()
            .skip(skip)
            .limit(limit)
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self
            .collection
            .find(filter)
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        let groups: Vec<StudyGroup> = cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        Ok((groups, total))
    }

    pub async fn find_by_creator(&self, user_id: &str) -> Result<Vec<StudyGroup>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "created_by": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn find_joined(&self, user_id: &str) -> Result<Vec<StudyGroup>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "members.user_id": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn update(&self, id: &str, update: bson::Document) -> Result<(), TeamderError> {
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": update })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        self.collection
            .delete_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn add_member(&self, id: &str, member: &GroupMember) -> Result<(), TeamderError> {
        let member_bson =
            bson::to_bson(member).map_err(|e| TeamderError::Database(e.to_string()))?;
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$push": { "members": member_bson },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_member(&self, id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$pull": { "members": { "user_id": user_id } },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn checkin(&self, id: &str, user_id: &str) -> Result<(), TeamderError> {
        let now = bson::DateTime::from_chrono(Utc::now());
        self.collection
            .update_one(
                doc! { "_id": id, "members.user_id": user_id },
                doc! {
                    "$set": {
                        "members.$.last_checkin": now,
                        "updated_at": now,
                    },
                    "$inc": { "members.$.streak": 1 },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn add_note(&self, id: &str, note: &StudyNote) -> Result<(), TeamderError> {
        let note_bson =
            bson::to_bson(note).map_err(|e| TeamderError::Database(e.to_string()))?;
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$push": { "notes": note_bson },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_note(&self, id: &str, note_id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$pull": { "notes": { "id": note_id } },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_progress(&self, id: &str, current_week: u8) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "current_week": current_week as i32,
                        "updated_at": bson::DateTime::from_chrono(Utc::now()),
                    }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn set_status(&self, id: &str, status: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "status": status, "updated_at": bson::DateTime::from_chrono(Utc::now()) } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.collection
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn search(&self, q: &str, limit: i64) -> Result<Vec<StudyGroup>, TeamderError> {
        let opts = FindOptions::builder().limit(limit).build();
        let filter = doc! {
            "name": { "$regex": q, "$options": "i" }
        };
        let cursor = self
            .collection
            .find(filter)
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
