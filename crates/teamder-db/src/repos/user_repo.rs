use futures_util::TryStreamExt;
use mongodb::{
    Collection,
    bson::{doc, Document},
};
use teamder_core::{
    error::TeamderError,
    models::user::{UpdateUserRequest, User},
};
use chrono::Utc;

use crate::DbClient;

#[derive(Clone)]
pub struct UserRepo {
    col: Collection<User>,
}

impl UserRepo {
    pub fn new(db: &DbClient) -> Self {
        Self {
            col: db.db.collection("users"),
        }
    }

    pub async fn create(&self, user: &User) -> Result<(), TeamderError> {
        self.col
            .insert_one(user)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, TeamderError> {
        let filter = doc! { "_id": id };
        self.col
            .find_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Batch-fetch users by IDs. Returns only the users that exist.
    pub async fn find_many_by_ids(&self, ids: &[&str]) -> Result<Vec<User>, TeamderError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let filter = doc! { "_id": { "$in": ids } };
        let cursor = self.col.find(filter).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, TeamderError> {
        let filter = doc! { "email": email };
        self.col
            .find_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(&self, limit: i64, skip: u64) -> Result<Vec<User>, TeamderError> {
        use mongodb::options::FindOptions;
        use futures_util::TryStreamExt;

        let opts = FindOptions::builder()
            .limit(limit)
            .skip(skip)
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self
            .col
            .find(doc! {})
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<User>, TeamderError> {
        use futures_util::TryStreamExt;
        let filter = doc! {
            "$or": [
                { "name": { "$regex": query, "$options": "i" } },
                { "role": { "$regex": query, "$options": "i" } },
                { "skill_tags": { "$regex": query, "$options": "i" } },
            ]
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

    pub async fn update(
        &self,
        id: &str,
        req: &UpdateUserRequest,
    ) -> Result<(), TeamderError> {
        use mongodb::bson::to_bson;

        let mut update_doc = Document::new();

        if let Some(v) = &req.name {
            update_doc.insert("name", v.clone());
        }
        if let Some(v) = &req.role {
            update_doc.insert("role", v.clone());
        }
        if let Some(v) = &req.department {
            update_doc.insert("department", v.clone());
        }
        if let Some(v) = &req.year {
            update_doc.insert("year", v.clone());
        }
        if let Some(v) = &req.location {
            update_doc.insert("location", v.clone());
        }
        if let Some(v) = &req.bio {
            let bio_bson: Vec<_> = v.iter().map(|s| mongodb::bson::Bson::String(s.clone())).collect();
            update_doc.insert("bio", bio_bson);
        }
        if let Some(v) = &req.skills {
            let bson = to_bson(v).map_err(|e| TeamderError::Database(e.to_string()))?;
            update_doc.insert("skills", bson);
        }
        if let Some(v) = &req.skill_tags {
            let tags: Vec<_> = v.iter().map(|s| mongodb::bson::Bson::String(s.clone())).collect();
            update_doc.insert("skill_tags", tags);
        }
        if let Some(v) = &req.work_mode {
            let bson = to_bson(v).map_err(|e| TeamderError::Database(e.to_string()))?;
            update_doc.insert("work_mode", bson);
        }
        if let Some(v) = &req.availability {
            let bson = to_bson(v).map_err(|e| TeamderError::Database(e.to_string()))?;
            update_doc.insert("availability", bson);
        }
        if let Some(v) = &req.hours_per_week {
            update_doc.insert("hours_per_week", v.clone());
        }
        if let Some(v) = &req.languages {
            let langs: Vec<_> = v.iter().map(|s| mongodb::bson::Bson::String(s.clone())).collect();
            update_doc.insert("languages", langs);
        }
        if let Some(v) = &req.portfolio {
            let bson = to_bson(v).map_err(|e| TeamderError::Database(e.to_string()))?;
            update_doc.insert("portfolio", bson);
        }
        // avatar_url uses double-Option: outer = "should we touch it", inner = value (None → null).
        if let Some(inner) = &req.avatar_url {
            match inner {
                Some(url) => update_doc.insert("avatar_url", url.clone()),
                None => update_doc.insert("avatar_url", mongodb::bson::Bson::Null),
            };
        }
        // resume_url uses double-Option: outer = "should we touch it", inner = value (None → null).
        if let Some(inner) = &req.resume_url {
            match inner {
                Some(url) => update_doc.insert("resume_url", url.clone()),
                None => update_doc.insert("resume_url", mongodb::bson::Bson::Null),
            };
        }
        if let Some(v) = &req.onboarded {
            update_doc.insert("onboarded", *v);
        }
        if let Some(inner) = &req.headline {
            match inner {
                Some(text) => update_doc.insert("headline", text.clone()),
                None => update_doc.insert("headline", mongodb::bson::Bson::Null),
            };
        }
        if let Some(v) = &req.notify_email {
            update_doc.insert("notify_email", *v);
        }
        if let Some(v) = &req.notify_in_app {
            update_doc.insert("notify_in_app", *v);
        }
        if let Some(v) = &req.is_public {
            update_doc.insert("is_public", *v);
        }
        if let Some(v) = &req.social_links {
            let bson = to_bson(v).map_err(|e| TeamderError::Database(e.to_string()))?;
            update_doc.insert("social_links", bson);
        }
        if let Some(v) = &req.interests {
            let arr: Vec<_> = v.iter().map(|s| mongodb::bson::Bson::String(s.clone())).collect();
            update_doc.insert("interests", arr);
        }
        if let Some(inner) = &req.timezone {
            match inner {
                Some(tz) => update_doc.insert("timezone", tz.clone()),
                None => update_doc.insert("timezone", mongodb::bson::Bson::Null),
            };
        }
        if let Some(inner) = &req.goals {
            match inner {
                Some(g) => update_doc.insert("goals", g.clone()),
                None => update_doc.insert("goals", mongodb::bson::Bson::Null),
            };
        }
        update_doc.insert("updated_at", Utc::now().to_rfc3339());

        let filter = doc! { "_id": id };
        let update = doc! { "$set": update_doc };

        self.col
            .update_one(filter, update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Set just the avatar_url field (used by the uploads route after saving an avatar image).
    pub async fn set_avatar_url(&self, id: &str, url: Option<String>) -> Result<(), TeamderError> {
        let value = match url {
            Some(u) => mongodb::bson::Bson::String(u),
            None => mongodb::bson::Bson::Null,
        };
        let filter = doc! { "_id": id };
        let update = doc! { "$set": { "avatar_url": value, "updated_at": Utc::now().to_rfc3339() } };
        self.col
            .update_one(filter, update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Set just the resume_url field (used by the uploads route after saving a file).
    pub async fn set_resume_url(&self, id: &str, url: Option<String>) -> Result<(), TeamderError> {
        let value = match url {
            Some(u) => mongodb::bson::Bson::String(u),
            None => mongodb::bson::Bson::Null,
        };
        let filter = doc! { "_id": id };
        let update = doc! { "$set": { "resume_url": value, "updated_at": Utc::now().to_rfc3339() } };
        self.col
            .update_one(filter, update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Append a portfolio item (used by uploads route after saving a file).
    pub async fn push_portfolio_item(
        &self,
        id: &str,
        item: &teamder_core::models::user::PortfolioItem,
    ) -> Result<(), TeamderError> {
        use mongodb::bson::to_bson;
        let bson = to_bson(item).map_err(|e| TeamderError::Database(e.to_string()))?;
        let filter = doc! { "_id": id };
        let update = doc! {
            "$push": { "portfolio": bson },
            "$set": { "updated_at": Utc::now().to_rfc3339() },
        };
        self.col
            .update_one(filter, update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), TeamderError> {
        let filter = doc! { "_id": id };
        self.col
            .delete_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Update the cached rating + review count on the user document.
    pub async fn set_rating(&self, id: &str, rating: f32, count: u32) -> Result<(), TeamderError> {
        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "rating": rating as f64,
                "collaborations": count as i64,
                "updated_at": Utc::now().to_rfc3339(),
            }
        };
        self.col
            .update_one(filter, update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Push an embedded Review entry (cached on the user document for quick display).
    pub async fn push_review(
        &self,
        user_id: &str,
        review: &teamder_core::models::user::Review,
    ) -> Result<(), TeamderError> {
        use mongodb::bson::to_bson;
        let bson = to_bson(review).map_err(|e| TeamderError::Database(e.to_string()))?;
        let filter = doc! { "_id": user_id };
        let update = doc! {
            "$push": { "reviews": bson },
            "$set": { "updated_at": Utc::now().to_rfc3339() },
        };
        self.col
            .update_one(filter, update)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Set a password reset token + expiry (or clear them by passing None).
    pub async fn set_reset_token(
        &self,
        id: &str,
        token: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), TeamderError> {
        let token_bson = match token {
            Some(t) => mongodb::bson::Bson::String(t.to_string()),
            None => mongodb::bson::Bson::Null,
        };
        let exp_bson = match expires_at {
            Some(e) => mongodb::bson::Bson::String(e.to_rfc3339()),
            None => mongodb::bson::Bson::Null,
        };
        self.col
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "reset_token": token_bson,
                        "reset_token_expires_at": exp_bson,
                        "updated_at": Utc::now().to_rfc3339(),
                    }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_reset_token(&self, token: &str) -> Result<Option<User>, TeamderError> {
        self.col
            .find_one(doc! { "reset_token": token })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn set_password_hash(&self, id: &str, hash: &str) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "password_hash": hash,
                        "reset_token": mongodb::bson::Bson::Null,
                        "reset_token_expires_at": mongodb::bson::Bson::Null,
                        "updated_at": Utc::now().to_rfc3339(),
                    }
                },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn set_admin(&self, id: &str, value: bool) -> Result<(), TeamderError> {
        self.col
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "is_admin": value, "updated_at": Utc::now().to_rfc3339() } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.col
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
