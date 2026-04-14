use mongodb::{Client, Database};
use anyhow::Result;

/// Wraps the MongoDB client and exposes typed collection access.
#[derive(Clone)]
pub struct DbClient {
    pub db: Database,
}

impl DbClient {
    /// Connect to MongoDB and return a ready `DbClient`.
    pub async fn connect(uri: &str, db_name: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(db_name);
        tracing::info!("Connected to MongoDB: {}", db_name);
        Ok(Self { db })
    }
}
