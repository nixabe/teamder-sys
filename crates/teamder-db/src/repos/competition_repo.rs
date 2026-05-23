use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{self, doc, Regex as BsonRegex};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::competition::{Competition, Registration};

pub struct CompetitionRepo {
    collection: Collection<Competition>,
}

impl CompetitionRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Competition>("competitions"),
        }
    }

    pub async fn create(&self, comp: &Competition) -> Result<(), TeamderError> {
        self.collection
            .insert_one(comp)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Competition>, TeamderError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(
        &self,
        status: Option<&str>,
        query: Option<&str>,
        skip: u64,
        limit: i64,
    ) -> Result<(Vec<Competition>, u64), TeamderError> {
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

        let comps: Vec<Competition> = cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        Ok((comps, total))
    }

    pub async fn featured(&self) -> Result<Vec<Competition>, TeamderError> {
        let filter = doc! {
            "is_featured": true,
            "publish_status": "published",
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

    pub async fn update(&self, id: &str, update: bson::Document) -> Result<(), TeamderError> {
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": update })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn register_user(
        &self,
        id: &str,
        registration: &Registration,
    ) -> Result<(), TeamderError> {
        let reg_bson =
            bson::to_bson(registration).map_err(|e| TeamderError::Database(e.to_string()))?;
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$push": { "registrations": reg_bson },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Toggle interest for a user. Returns `true` if now interested, `false` if removed.
    pub async fn toggle_interest(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<bool, TeamderError> {
        // Check if user is already interested
        let comp = self.find_by_id(id).await?;
        let comp = comp.ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

        let is_interested = comp.interested_user_ids.iter().any(|uid| uid == user_id);

        if is_interested {
            // Remove interest
            self.collection
                .update_one(
                    doc! { "_id": id },
                    doc! {
                        "$pull": { "interested_user_ids": user_id },
                        "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                    },
                )
                .await
                .map_err(|e| TeamderError::Database(e.to_string()))?;
            Ok(false)
        } else {
            // Add interest
            self.collection
                .update_one(
                    doc! { "_id": id },
                    doc! {
                        "$push": { "interested_user_ids": user_id },
                        "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                    },
                )
                .await
                .map_err(|e| TeamderError::Database(e.to_string()))?;
            Ok(true)
        }
    }

    pub async fn set_winners(&self, id: &str, winner_ids: &[String]) -> Result<(), TeamderError> {
        let winners_bson: Vec<bson::Bson> =
            winner_ids.iter().map(|w| bson::Bson::String(w.clone())).collect();
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "winners": winners_bson,
                        "updated_at": bson::DateTime::from_chrono(Utc::now()),
                    }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn set_publish_status(
        &self,
        id: &str,
        status: &str,
        note: Option<&str>,
    ) -> Result<(), TeamderError> {
        let mut set = doc! {
            "publish_status": status,
            "updated_at": bson::DateTime::from_chrono(Utc::now()),
        };
        if let Some(n) = note {
            set.insert("rejected_note", n);
        }
        self.collection
            .update_one(doc! { "_id": id }, doc! { "$set": set })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_publisher(
        &self,
        publisher_id: &str,
    ) -> Result<Vec<Competition>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "publisher_id": publisher_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn find_pending(&self) -> Result<Vec<Competition>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "publish_status": "pending_review" })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.collection
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn search(&self, q: &str, limit: i64) -> Result<Vec<Competition>, TeamderError> {
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
