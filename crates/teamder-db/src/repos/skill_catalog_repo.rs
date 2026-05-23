use futures::TryStreamExt;
use mongodb::bson::{self, doc};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use teamder_core::error::TeamderError;
use teamder_core::models::skill_catalog::{StoredSkillCategory, StoredSkillTag};

pub struct SkillCatalogRepo {
    categories: Collection<StoredSkillCategory>,
    tags: Collection<StoredSkillTag>,
}

impl SkillCatalogRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            categories: db.collection::<StoredSkillCategory>("skill_categories"),
            tags: db.collection::<StoredSkillTag>("skill_tags"),
        }
    }

    pub async fn list_categories(&self) -> Result<Vec<StoredSkillCategory>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "order": 1 })
            .build();
        let cursor = self
            .categories
            .find(doc! {})
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_tags(&self) -> Result<Vec<StoredSkillTag>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "category_key": 1, "order": 1 })
            .build();
        let cursor = self
            .tags
            .find(doc! {})
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_tags_by_category(
        &self,
        category_key: &str,
    ) -> Result<Vec<StoredSkillTag>, TeamderError> {
        let opts = FindOptions::builder()
            .sort(doc! { "order": 1 })
            .build();
        let cursor = self
            .tags
            .find(doc! { "category_key": category_key })
            .with_options(opts)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn create_category(
        &self,
        cat: &StoredSkillCategory,
    ) -> Result<(), TeamderError> {
        self.categories
            .insert_one(cat)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_category(
        &self,
        key: &str,
        update: bson::Document,
    ) -> Result<(), TeamderError> {
        self.categories
            .update_one(doc! { "_id": key }, doc! { "$set": update })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_category(&self, key: &str) -> Result<(), TeamderError> {
        self.categories
            .delete_one(doc! { "_id": key })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn create_tag(&self, tag: &StoredSkillTag) -> Result<(), TeamderError> {
        self.tags
            .insert_one(tag)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_tag(
        &self,
        id: &str,
        update: bson::Document,
    ) -> Result<(), TeamderError> {
        self.tags
            .update_one(doc! { "_id": id }, doc! { "$set": update })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_tag(&self, id: &str) -> Result<(), TeamderError> {
        self.tags
            .delete_one(doc! { "_id": id })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn count_categories(&self) -> Result<u64, TeamderError> {
        self.categories
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn count_tags(&self) -> Result<u64, TeamderError> {
        self.tags
            .count_documents(doc! {})
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }
}
