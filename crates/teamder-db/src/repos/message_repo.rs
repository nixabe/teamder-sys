use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::bson::{self, doc};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use teamder_core::error::TeamderError;
use teamder_core::models::message::Message;

/// Summary of a conversation with a partner user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub partner_id: String,
    pub partner_name: String,
    pub last_message: String,
    pub unread_count: i64,
    pub updated_at: DateTime<Utc>,
}

pub struct MessageRepo {
    collection: Collection<Message>,
}

impl MessageRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Message>("messages"),
        }
    }

    pub async fn create(&self, msg: &Message) -> Result<(), TeamderError> {
        self.collection
            .insert_one(msg)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get conversation summaries for a user.
    ///
    /// Uses aggregation pipeline to group messages by conversation partner,
    /// extracting the last message text and unread count.
    pub async fn conversations(
        &self,
        user_id: &str,
    ) -> Result<Vec<ConversationSummary>, TeamderError> {
        let pipeline = vec![
            // Match messages involving this user
            doc! {
                "$match": {
                    "$or": [
                        { "from_user_id": user_id },
                        { "to_user_id": user_id },
                    ]
                }
            },
            // Sort by created_at desc so $first picks the latest
            doc! { "$sort": { "created_at": -1 } },
            // Add a partner_id field
            doc! {
                "$addFields": {
                    "partner_id": {
                        "$cond": {
                            "if": { "$eq": ["$from_user_id", user_id] },
                            "then": "$to_user_id",
                            "else": "$from_user_id",
                        }
                    }
                }
            },
            // Group by partner
            doc! {
                "$group": {
                    "_id": "$partner_id",
                    "last_message": { "$first": "$content" },
                    "updated_at": { "$first": "$created_at" },
                    "unread_count": {
                        "$sum": {
                            "$cond": {
                                "if": {
                                    "$and": [
                                        { "$eq": ["$to_user_id", user_id] },
                                        { "$eq": ["$read", false] },
                                    ]
                                },
                                "then": 1,
                                "else": 0,
                            }
                        }
                    }
                }
            },
            // Sort by most recent conversation
            doc! { "$sort": { "updated_at": -1 } },
            // Project into the shape we want
            doc! {
                "$project": {
                    "_id": 0,
                    "partner_id": "$_id",
                    "partner_name": { "$literal": "" },
                    "last_message": 1,
                    "unread_count": 1,
                    "updated_at": 1,
                }
            },
        ];

        let raw_coll = self
            .collection
            .clone_with_type::<bson::Document>();

        let cursor = raw_coll
            .aggregate(pipeline)
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        let docs: Vec<bson::Document> = cursor
            .try_collect()
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;

        let mut summaries = Vec::new();
        for d in docs {
            let summary: ConversationSummary =
                bson::from_document(d).map_err(|e| TeamderError::Database(e.to_string()))?;
            summaries.push(summary);
        }

        Ok(summaries)
    }

    pub async fn messages_with(
        &self,
        user_id: &str,
        partner_id: &str,
    ) -> Result<Vec<Message>, TeamderError> {
        let filter = doc! {
            "$or": [
                { "from_user_id": user_id, "to_user_id": partner_id },
                { "from_user_id": partner_id, "to_user_id": user_id },
            ]
        };
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": 1 })
            .build();

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

    /// Mark all messages from a partner to this user as read.
    pub async fn mark_read(&self, from_id: &str, to_id: &str) -> Result<(), TeamderError> {
        self.collection
            .update_many(
                doc! { "from_user_id": from_id, "to_user_id": to_id, "read": false },
                doc! { "$set": { "read": true } },
            )
            .await
            .map_err(|e| TeamderError::Database(e.to_string()))?;
        Ok(())
    }
}
