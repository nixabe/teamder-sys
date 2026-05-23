use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::invite::Invite;

pub struct InviteRepo {
    collection: Collection<Invite>,
}

impl InviteRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Invite>("invites"),
        }
    }

    pub async fn create(&self, invite: &Invite) -> Result<(), TeamderError> {
        self.collection
            .insert_one(invite)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Invite>, TeamderError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Check for a duplicate pending invite between two users for the same context.
    pub async fn find_pending_between(
        &self,
        from_id: &str,
        to_id: &str,
        project_id: Option<&str>,
        study_group_id: Option<&str>,
        competition_team_id: Option<&str>,
    ) -> Result<Option<Invite>, TeamderError> {
        let mut filter = doc! {
            "from_user_id": from_id,
            "to_user_id": to_id,
            "status": "pending",
        };

        match (project_id, study_group_id, competition_team_id) {
            (Some(pid), _, _) => {
                filter.insert("project_id", pid);
            }
            (_, Some(sgid), _) => {
                filter.insert("study_group_id", sgid);
            }
            (_, _, Some(ctid)) => {
                filter.insert("competition_team_id", ctid);
            }
            _ => {}
        }

        self.collection
            .find_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<Invite>, TeamderError> {
        let filter = doc! {
            "$or": [
                { "to_user_id": user_id },
                { "from_user_id": user_id },
            ]
        };
        let cursor = self
            .collection
            .find(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn update_status(&self, id: &str, status: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": { "status": status } })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn mark_read(&self, id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": { "is_read": true } })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn mark_all_read(&self, user_id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_many(
                doc! { "to_user_id": user_id, "is_read": false },
                doc! { "$set": { "is_read": true } },
            )
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

    pub async fn count_pending(&self, user_id: &str) -> Result<u64, TeamderError> {
        self.collection
            .count_documents(doc! { "to_user_id": user_id, "status": "pending" })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
