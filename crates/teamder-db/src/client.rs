use mongodb::{Client, Database};

use crate::repos::{
    BookmarkRepo, CompetitionRepo, CompetitionTeamRepo, InviteRepo, JoinRequestRepo, MessageRepo,
    NotificationRepo, PeerReviewRepo, ProjectRepo, ProjectUpdateRepo, SkillCatalogRepo,
    StudyGroupRepo, UserRepo,
};

/// Wrapper around a MongoDB `Database` handle.
///
/// Constructed once at startup and shared (via Rocket managed state) with all
/// request handlers.
#[derive(Clone, Debug)]
pub struct DbClient {
    db: Database,
}

impl DbClient {
    /// Connect to MongoDB and select the given database.
    pub async fn new(uri: &str, db_name: &str) -> anyhow::Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(db_name);

        // Ping to verify the connection is alive.
        db.run_command(mongodb::bson::doc! { "ping": 1 })
            .await?;

        tracing::info!("Connected to MongoDB database '{}'", db_name);

        Ok(Self { db })
    }

    /// Returns a reference to the underlying `mongodb::Database`.
    pub fn database(&self) -> &Database {
        &self.db
    }

    // ── Repo accessors ──────────────────────────────────────────────────────

    pub fn user_repo(&self) -> UserRepo {
        UserRepo::new(&self.db)
    }

    pub fn project_repo(&self) -> ProjectRepo {
        ProjectRepo::new(&self.db)
    }

    pub fn competition_repo(&self) -> CompetitionRepo {
        CompetitionRepo::new(&self.db)
    }

    pub fn competition_team_repo(&self) -> CompetitionTeamRepo {
        CompetitionTeamRepo::new(&self.db)
    }

    pub fn study_group_repo(&self) -> StudyGroupRepo {
        StudyGroupRepo::new(&self.db)
    }

    pub fn invite_repo(&self) -> InviteRepo {
        InviteRepo::new(&self.db)
    }

    pub fn join_request_repo(&self) -> JoinRequestRepo {
        JoinRequestRepo::new(&self.db)
    }

    pub fn peer_review_repo(&self) -> PeerReviewRepo {
        PeerReviewRepo::new(&self.db)
    }

    pub fn message_repo(&self) -> MessageRepo {
        MessageRepo::new(&self.db)
    }

    pub fn notification_repo(&self) -> NotificationRepo {
        NotificationRepo::new(&self.db)
    }

    pub fn bookmark_repo(&self) -> BookmarkRepo {
        BookmarkRepo::new(&self.db)
    }

    pub fn skill_catalog_repo(&self) -> SkillCatalogRepo {
        SkillCatalogRepo::new(&self.db)
    }

    pub fn project_update_repo(&self) -> ProjectUpdateRepo {
        ProjectUpdateRepo::new(&self.db)
    }
}
