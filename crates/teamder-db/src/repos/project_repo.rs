use futures_util::TryStreamExt;
use mongodb::{
    Collection,
    bson::{doc, Document},
};
use teamder_core::{
    error::TeamderError,
    models::project::{Project, TeamMember, UpdateProjectRequest},
};
use chrono::Utc;

use crate::DbClient;

pub struct ProjectRepo {
    col: Collection<Project>,
}

impl ProjectRepo {
    pub fn new(db: &DbClient) -> Self {
        Self {
            col: db.db.collection("projects"),
        }
    }

    pub async fn create(&self, project: &Project) -> Result<(), TeamderError> {
        self.col
            .insert_one(project)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Project>, TeamderError> {
        self.col
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(&self, limit: i64, skip: u64) -> Result<Vec<Project>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder()
            .limit(limit)
            .skip(skip)
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self
            .col
            .find(doc! { "is_public": true })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_by_status(&self, status: &str) -> Result<Vec<Project>, TeamderError> {
        let cursor = self
            .col
            .find(doc! { "status": status, "is_public": true })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_by_member(&self, user_id: &str) -> Result<Vec<Project>, TeamderError> {
        let cursor = self
            .col
            .find(doc! { "team.user_id": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_by_lead(&self, user_id: &str) -> Result<Vec<Project>, TeamderError> {
        let cursor = self
            .col
            .find(doc! { "lead_user_id": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Project>, TeamderError> {
        let filter = doc! {
            "$or": [
                { "name": { "$regex": query, "$options": "i" } },
                { "description": { "$regex": query, "$options": "i" } },
                { "skills": { "$regex": query, "$options": "i" } },
            ],
            "is_public": true
        };
        let cursor = self
            .col
            .find(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn update(&self, id: &str, req: &UpdateProjectRequest) -> Result<(), TeamderError> {
        let mut update_doc = Document::new();
        if let Some(v) = &req.name { update_doc.insert("name", v.clone()); }
        if let Some(v) = &req.description { update_doc.insert("description", v.clone()); }
        if let Some(v) = &req.goals { update_doc.insert("goals", v.clone()); }
        if let Some(v) = &req.deadline { update_doc.insert("deadline", v.clone()); }
        if let Some(v) = &req.duration { update_doc.insert("duration", v.clone()); }
        if let Some(v) = &req.is_public { update_doc.insert("is_public", *v); }
        if let Some(v) = &req.status {
            let bson = mongodb::bson::to_bson(v)
                .map_err(|e| TeamderError::Internal(e.to_string()))?;
            update_doc.insert("status", bson);
        }
        if let Some(v) = &req.collab {
            let bson = mongodb::bson::to_bson(v)
                .map_err(|e| TeamderError::Internal(e.to_string()))?;
            update_doc.insert("collab", bson);
        }
        if let Some(v) = &req.roles {
            let bson = mongodb::bson::to_bson(v)
                .map_err(|e| TeamderError::Internal(e.to_string()))?;
            update_doc.insert("roles", bson);
        }
        if let Some(v) = &req.skills {
            let bson = mongodb::bson::to_bson(v)
                .map_err(|e| TeamderError::Internal(e.to_string()))?;
            update_doc.insert("skills", bson);
        }
        if let Some(v) = &req.join_mode {
            let bson = mongodb::bson::to_bson(v)
                .map_err(|e| TeamderError::Internal(e.to_string()))?;
            update_doc.insert("join_mode", bson);
        }
        update_doc.insert("updated_at", Utc::now().to_rfc3339());

        self.col
            .update_one(doc! { "_id": id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn add_member(&self, project_id: &str, member: &TeamMember) -> Result<(), TeamderError> {
        let member_bson = mongodb::bson::to_bson(member)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col
            .update_one(
                doc! { "_id": project_id },
                doc! { "$push": { "team": member_bson } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_member(&self, project_id: &str, user_id: &str) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": project_id },
                doc! { "$pull": { "team": { "user_id": user_id } } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        self.col
            .delete_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Toggle the is_promoted flag (admin or owner action).
    pub async fn set_promoted(&self, id: &str, value: bool) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "is_promoted": value, "updated_at": Utc::now().to_rfc3339() } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// List projects sorted with promoted ones first.
    pub async fn list_with_promotion(&self, limit: i64, skip: u64) -> Result<Vec<Project>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder()
            .limit(limit)
            .skip(skip)
            .sort(doc! { "is_promoted": -1, "created_at": -1 })
            .build();
        let cursor = self
            .col
            .find(doc! { "is_public": true })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.col
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
