use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{self, doc};
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::competition_team::{CompTeamMember, CompetitionTeam};

pub struct CompetitionTeamRepo {
    collection: Collection<CompetitionTeam>,
}

impl CompetitionTeamRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<CompetitionTeam>("competition_teams"),
        }
    }

    pub async fn create(&self, team: &CompetitionTeam) -> Result<(), TeamderError> {
        self.collection
            .insert_one(team)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<CompetitionTeam>, TeamderError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_by_competition(
        &self,
        competition_id: &str,
    ) -> Result<Vec<CompetitionTeam>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "competition_id": competition_id })
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

    pub async fn add_member(
        &self,
        id: &str,
        member: &CompTeamMember,
    ) -> Result<(), TeamderError> {
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

    pub async fn find_by_lead(
        &self,
        user_id: &str,
    ) -> Result<Vec<CompetitionTeam>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "lead_user_id": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
