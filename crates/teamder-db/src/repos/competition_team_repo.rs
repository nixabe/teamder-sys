use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use teamder_core::{
    error::TeamderError,
    models::competition_team::{CompetitionTeam, CompetitionTeamMember, CompetitionTeamStatus, UpdateCompetitionTeamRequest},
};

use crate::DbClient;

#[derive(Clone)]
pub struct CompetitionTeamRepo {
    col: Collection<CompetitionTeam>,
}

impl CompetitionTeamRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("competition_teams") }
    }

    pub async fn create(&self, t: &CompetitionTeam) -> Result<(), TeamderError> {
        self.col.insert_one(t).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<CompetitionTeam>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_for_competition(&self, competition_id: &str) -> Result<Vec<CompetitionTeam>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! { "competition_id": competition_id })
            .with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<CompetitionTeam>, TeamderError> {
        let cursor = self.col.find(doc! { "members.user_id": user_id })
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn add_member(&self, team_id: &str, member: &CompetitionTeamMember) -> Result<(), TeamderError> {
        let bson = mongodb::bson::to_bson(member).map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col.update_one(
            doc! { "_id": team_id },
            doc! { "$push": { "members": bson }, "$set": { "updated_at": Utc::now().to_rfc3339() } },
        ).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_member(&self, team_id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.col.update_one(
            doc! { "_id": team_id },
            doc! { "$pull": { "members": { "user_id": user_id } }, "$set": { "updated_at": Utc::now().to_rfc3339() } },
        ).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update(&self, id: &str, req: &UpdateCompetitionTeamRequest) -> Result<(), TeamderError> {
        use mongodb::bson::Document;
        let mut update_doc = Document::new();
        if let Some(v) = &req.name { update_doc.insert("name", v.clone()); }
        if let Some(v) = &req.description { update_doc.insert("description", v.clone()); }
        if let Some(v) = req.max_members { update_doc.insert("max_members", v as i32); }
        if let Some(v) = &req.looking_for {
            let arr: Vec<_> = v.iter().map(|s| mongodb::bson::Bson::String(s.clone())).collect();
            update_doc.insert("looking_for", arr);
        }
        if let Some(v) = &req.open_roles {
            let arr: Vec<_> = v.iter().map(|s| mongodb::bson::Bson::String(s.clone())).collect();
            update_doc.insert("open_roles", arr);
        }
        if let Some(v) = &req.status {
            let bson = mongodb::bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?;
            update_doc.insert("status", bson);
        }
        update_doc.insert("updated_at", Utc::now().to_rfc3339());
        self.col.update_one(doc! { "_id": id }, doc! { "$set": update_doc })
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn set_status(&self, id: &str, status: CompetitionTeamStatus) -> Result<(), TeamderError> {
        let bson = mongodb::bson::to_bson(&status).map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col.update_one(
            doc! { "_id": id },
            doc! { "$set": { "status": bson, "updated_at": Utc::now().to_rfc3339() } },
        ).await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        self.col.delete_one(doc! { "_id": id })
            .await.map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
