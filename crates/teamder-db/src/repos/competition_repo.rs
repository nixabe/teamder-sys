use futures_util::TryStreamExt;
use mongodb::{Collection, bson::doc};
use teamder_core::{error::TeamderError, models::competition::Competition};
use crate::DbClient;

pub struct CompetitionRepo {
    col: Collection<Competition>,
}

impl CompetitionRepo {
    pub fn new(db: &DbClient) -> Self {
        Self { col: db.db.collection("competitions") }
    }

    pub async fn create(&self, comp: &Competition) -> Result<(), TeamderError> {
        self.col.insert_one(comp).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Competition>, TeamderError> {
        self.col.find_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list(&self) -> Result<Vec<Competition>, TeamderError> {
        use mongodb::options::FindOptions;
        let opts = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.col.find(doc! {}).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_featured(&self) -> Result<Vec<Competition>, TeamderError> {
        let cursor = self.col.find(doc! { "is_featured": true }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn add_registration(
        &self,
        comp_id: &str,
        reg: &teamder_core::models::competition::Registration,
    ) -> Result<(), TeamderError> {
        let reg_bson = mongodb::bson::to_bson(reg)
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
        self.col
            .update_one(
                doc! { "_id": comp_id },
                doc! { "$push": { "registrations": reg_bson } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count(&self) -> Result<u64, TeamderError> {
        self.col.count_documents(doc! {}).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
