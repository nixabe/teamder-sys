use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::{
    Collection,
    bson::{doc, Document},
    options::FindOptions,
};
use teamder_core::{
    error::TeamderError,
    models::skill_catalog::{
        StoredSkillCategory, StoredSkillTag, UpdateCategoryRequest, UpdateTagRequest,
    },
};

use crate::DbClient;

#[derive(Clone)]
pub struct SkillCatalogRepo {
    cats: Collection<StoredSkillCategory>,
    tags: Collection<StoredSkillTag>,
}

impl SkillCatalogRepo {
    pub fn new(db: &DbClient) -> Self {
        Self {
            cats: db.db.collection("skill_categories"),
            tags: db.db.collection("skill_tags"),
        }
    }

    // ── categories ─────────────────────────────────────────────

    pub async fn list_categories(&self) -> Result<Vec<StoredSkillCategory>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "order": 1, "label": 1 }).build();
        let cursor = self.cats.find(doc! {}).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn count_categories(&self) -> Result<u64, TeamderError> {
        self.cats.count_documents(doc! {}).await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn category_exists(&self, key: &str) -> Result<bool, TeamderError> {
        let n = self.cats.count_documents(doc! { "_id": key }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(n > 0)
    }

    pub async fn insert_category(&self, cat: &StoredSkillCategory) -> Result<(), TeamderError> {
        self.cats.insert_one(cat).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_category(&self, key: &str, req: &UpdateCategoryRequest) -> Result<(), TeamderError> {
        let mut doc_set = Document::new();
        if let Some(v) = &req.label { doc_set.insert("label", v.clone()); }
        if let Some(v) = &req.label_zh { doc_set.insert("label_zh", v.clone()); }
        if let Some(v) = req.order { doc_set.insert("order", v); }
        doc_set.insert("updated_at", Utc::now().to_rfc3339());
        self.cats.update_one(doc! { "_id": key }, doc! { "$set": doc_set }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_category(&self, key: &str) -> Result<(), TeamderError> {
        // Cascade: drop all tags in the category too.
        self.tags.delete_many(doc! { "category_key": key }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        self.cats.delete_one(doc! { "_id": key }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    // ── tags ───────────────────────────────────────────────────

    pub async fn list_tags(&self) -> Result<Vec<StoredSkillTag>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "category_key": 1, "order": 1, "name": 1 }).build();
        let cursor = self.tags.find(doc! {}).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn list_active_tags(&self) -> Result<Vec<StoredSkillTag>, TeamderError> {
        let opts = FindOptions::builder().sort(doc! { "category_key": 1, "order": 1, "name": 1 }).build();
        let cursor = self.tags.find(doc! { "is_active": true }).with_options(opts).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        cursor.try_collect().await.map_err(|e| TeamderError::Database(e.to_string()))
    }

    pub async fn insert_tag(&self, tag: &StoredSkillTag) -> Result<(), TeamderError> {
        self.tags.insert_one(tag).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn update_tag(&self, id: &str, req: &UpdateTagRequest) -> Result<(), TeamderError> {
        let mut doc_set = Document::new();
        if let Some(v) = &req.name { doc_set.insert("name", v.clone()); }
        if let Some(v) = &req.name_zh { doc_set.insert("name_zh", v.clone()); }
        if let Some(v) = &req.category_key { doc_set.insert("category_key", v.clone()); }
        if let Some(v) = req.order { doc_set.insert("order", v); }
        if let Some(v) = req.is_active { doc_set.insert("is_active", v); }
        doc_set.insert("updated_at", Utc::now().to_rfc3339());
        self.tags.update_one(doc! { "_id": id }, doc! { "$set": doc_set }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_tag(&self, id: &str) -> Result<(), TeamderError> {
        self.tags.delete_one(doc! { "_id": id }).await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
