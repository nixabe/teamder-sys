use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::peer_review::PeerReview;

pub struct PeerReviewRepo {
    collection: Collection<PeerReview>,
}

impl PeerReviewRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<PeerReview>("peer_reviews"),
        }
    }

    pub async fn create(&self, review: &PeerReview) -> Result<(), TeamderError> {
        self.collection
            .insert_one(review)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_for_user(&self, user_id: &str) -> Result<Vec<PeerReview>, TeamderError> {
        let cursor = self
            .collection
            .find(doc! { "reviewee_id": user_id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
