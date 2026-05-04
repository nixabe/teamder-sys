use futures_util::TryStreamExt;
use mongodb::{
    Collection,
    bson::doc,
    options::FindOptions,
};
use teamder_core::{
    error::TeamderError,
    models::peer_review::PeerReview,
};

use crate::DbClient;

#[derive(Clone)]
pub struct PeerReviewRepo {
    col: Collection<PeerReview>,
}

impl PeerReviewRepo {
    pub fn new(db: &DbClient) -> Self {
        Self {
            col: db.db.collection("peer_reviews"),
        }
    }

    pub async fn create(&self, review: &PeerReview) -> Result<(), TeamderError> {
        self.col
            .insert_one(review)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Reviews left FOR a user (reviewee).
    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<PeerReview>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(50)
            .build();
        let cursor = self
            .col
            .find(doc! { "reviewee_id": user_id })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Reviews written BY a user (reviewer).
    pub async fn list_by_reviewer(&self, user_id: &str) -> Result<Vec<PeerReview>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();
        let cursor = self
            .col
            .find(doc! { "reviewer_id": user_id })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Returns true if reviewer has already reviewed reviewee on this project.
    pub async fn exists_pair(
        &self,
        reviewer_id: &str,
        reviewee_id: &str,
        project_id: Option<&str>,
    ) -> Result<bool, TeamderError> {
        let mut filter = doc! {
            "reviewer_id": reviewer_id,
            "reviewee_id": reviewee_id,
        };
        if let Some(pid) = project_id {
            filter.insert("project_id", pid);
        } else {
            filter.insert("project_id", mongodb::bson::Bson::Null);
        }
        let found = self
            .col
            .find_one(filter)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(found.is_some())
    }

    /// Average rating of all reviews for a user (1.0–5.0). Returns 0 if no reviews.
    pub async fn average_for_user(&self, user_id: &str) -> Result<(f32, u32), TeamderError> {
        let reviews = self.list_for_user(user_id).await?;
        if reviews.is_empty() {
            return Ok((0.0, 0));
        }
        let n = reviews.len() as f32;
        let sum: f32 = reviews.iter().map(|r| r.scores.average()).sum();
        Ok((sum / n, reviews.len() as u32))
    }
}
