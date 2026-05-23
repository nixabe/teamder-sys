use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::bson::{self, doc, Regex as BsonRegex};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::user::{CachedReview, PortfolioItem, User};

pub struct UserRepo {
    collection: Collection<User>,
}

impl UserRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<User>("users"),
        }
    }

    pub async fn create(&self, user: &User) -> Result<(), TeamderError> {
        self.collection
            .insert_one(user)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, TeamderError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, TeamderError> {
        self.collection
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(
        &self,
        skip: u64,
        limit: i64,
        query: Option<&str>,
    ) -> Result<(Vec<User>, u64), TeamderError> {
        let filter = match query {
            Some(q) if !q.is_empty() => {
                let regex = BsonRegex {
                    pattern: q.to_string(),
                    options: "i".to_string(),
                };
                doc! {
                    "$or": [
                        { "name": { "$regex": &regex.pattern, "$options": &regex.options } },
                        { "email": { "$regex": &regex.pattern, "$options": &regex.options } },
                    ]
                }
            }
            _ => doc! {},
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

        let users: Vec<User> = cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        Ok((users, total))
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

    pub async fn set_reset_token(
        &self,
        email: &str,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "reset_token": token,
                        "reset_token_expires_at": bson::DateTime::from_chrono(expires_at),
                    }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_reset_token(&self, token: &str) -> Result<Option<User>, TeamderError> {
        self.collection
            .find_one(doc! { "reset_token": token })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn clear_reset_token(&self, id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "reset_token": bson::Bson::Null,
                        "reset_token_expires_at": bson::Bson::Null,
                    }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_avatar(&self, id: &str, url: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "avatar_url": url, "updated_at": bson::DateTime::from_chrono(Utc::now()) } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_resume(&self, id: &str, url: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "resume_url": url, "updated_at": bson::DateTime::from_chrono(Utc::now()) } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn append_portfolio(
        &self,
        id: &str,
        item: &PortfolioItem,
    ) -> Result<(), TeamderError> {
        let item_bson =
            bson::to_bson(item).map_err(|e| TeamderError::Database(e.to_string()))?;
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$push": { "portfolio": item_bson },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_portfolio(&self, id: &str, title: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$pull": { "portfolio": { "title": title } },
                    "$set": { "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn mark_onboarded(&self, id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "onboarded": true, "updated_at": bson::DateTime::from_chrono(Utc::now()) } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_rating(
        &self,
        id: &str,
        rating: f32,
        review: &CachedReview,
    ) -> Result<(), TeamderError> {
        let review_bson =
            bson::to_bson(review).map_err(|e| TeamderError::Database(e.to_string()))?;
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": { "rating": rating, "updated_at": bson::DateTime::from_chrono(Utc::now()) },
                    "$push": { "reviews": review_bson },
                },
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

    pub async fn search(&self, q: &str, limit: i64) -> Result<Vec<User>, TeamderError> {
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
