use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{self, doc, Regex as BsonRegex};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::project::{Project, TeamMember};

pub struct ProjectRepo {
    collection: Collection<Project>,
}

impl ProjectRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Project>("projects"),
        }
    }

    pub async fn create(&self, project: &Project) -> Result<(), TeamderError> {
        self.collection
            .insert_one(project)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Project>, TeamderError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(
        &self,
        skip: u64,
        limit: i64,
        status: Option<&str>,
        query: Option<&str>,
    ) -> Result<(Vec<Project>, u64), TeamderError> {
        let mut filter = doc! {};

        if let Some(s) = status {
            filter.insert("status", s);
        }

        if let Some(q) = query {
            if !q.is_empty() {
                let regex = BsonRegex {
                    pattern: q.to_string(),
                    options: "i".to_string(),
                };
                filter.insert(
                    "$or",
                    bson::bson!([
                        { "name": { "$regex": &regex.pattern, "$options": &regex.options } },
                        { "description": { "$regex": &regex.pattern, "$options": &regex.options } },
                    ]),
                );
            }
        }

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

        let projects: Vec<Project> = cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        Ok((projects, total))
    }

    pub async fn find_by_lead(&self, user_id: &str) -> Result<Vec<Project>, TeamderError> {
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

    pub async fn find_joined(&self, user_id: &str) -> Result<Vec<Project>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "team.user_id": user_id })
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

    pub async fn add_member(&self, id: &str, member: &TeamMember) -> Result<(), TeamderError> {
        let member_bson =
            bson::to_bson(member).map_err(|e| TeamderError::Database(e.to_string()))?;
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$push": { "team": member_bson },
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
                    "$pull": { "team": { "user_id": user_id } },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn set_member_role(
        &self,
        id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id, "team.user_id": user_id },
                doc! {
                    "$set": {
                        "team.$.role": role,
                        "updated_at": bson::DateTime::from_chrono(Utc::now()),
                    }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn increment_role_filled(
        &self,
        id: &str,
        role_name: &str,
    ) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id, "roles.name": role_name },
                doc! {
                    "$inc": { "roles.$.filled": 1 },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn decrement_role_filled(
        &self,
        id: &str,
        role_name: &str,
    ) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id, "roles.name": role_name },
                doc! {
                    "$inc": { "roles.$.filled": -1 },
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

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.collection
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn search(&self, q: &str, limit: i64) -> Result<Vec<Project>, TeamderError> {
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

    pub async fn list_all(&self) -> Result<Vec<Project>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
