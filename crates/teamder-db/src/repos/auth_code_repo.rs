use mongodb::{bson::doc, Collection};
use teamder_core::{error::TeamderError, models::auth_code::AuthCode};

use crate::DbClient;

#[derive(Clone)]
pub struct AuthCodeRepo {
    col: Collection<AuthCode>,
}

impl AuthCodeRepo {
    pub fn new(db: &DbClient) -> Self {
        Self {
            col: db.db.collection("auth_codes"),
        }
    }

    /// Replace any existing code for this (email, purpose) with a new one.
    pub async fn set_code(&self, code: &AuthCode) -> Result<(), TeamderError> {
        self.delete(&code.email, &code.purpose).await?;
        self.col
            .insert_one(code)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Find the active code for a given (email, purpose), if any.
    pub async fn find(&self, email: &str, purpose: &str) -> Result<Option<AuthCode>, TeamderError> {
        self.col
            .find_one(doc! { "email": email, "purpose": purpose })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))
    }

    /// Remove all codes for a (email, purpose) pair (after use or before re-issuing).
    pub async fn delete(&self, email: &str, purpose: &str) -> Result<(), TeamderError> {
        self.col
            .delete_many(doc! { "email": email, "purpose": purpose })
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
