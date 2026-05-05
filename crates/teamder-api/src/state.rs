use teamder_db::{
    DbClient,
    repos::{
        BookmarkRepo, CompetitionRepo, CompetitionTeamRepo, InviteRepo, JoinRequestRepo,
        MessageRepo, NotificationRepo, PeerReviewRepo, ProjectRepo, ProjectUpdateRepo,
        SkillCatalogRepo, StudyGroupRepo, UserRepo,
    },
};

use crate::chat::ChatState;

/// Shared application state injected into every Rocket handler.
pub struct AppState {
    pub users: UserRepo,
    pub projects: ProjectRepo,
    pub competitions: CompetitionRepo,
    pub study_groups: StudyGroupRepo,
    pub invites: InviteRepo,
    pub messages: MessageRepo,
    pub join_requests: JoinRequestRepo,
    pub peer_reviews: PeerReviewRepo,
    pub notifications: NotificationRepo,
    pub competition_teams: CompetitionTeamRepo,
    pub bookmarks: BookmarkRepo,
    pub project_updates: ProjectUpdateRepo,
    pub skill_catalog: SkillCatalogRepo,
    pub chat: ChatState,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db: DbClient) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "teamder-dev-secret-change-in-production".to_string());
        Self::new_with_secret(db, jwt_secret)
    }

    pub fn new_with_secret(db: DbClient, jwt_secret: String) -> Self {
        Self {
            users: UserRepo::new(&db),
            projects: ProjectRepo::new(&db),
            competitions: CompetitionRepo::new(&db),
            study_groups: StudyGroupRepo::new(&db),
            invites: InviteRepo::new(&db),
            messages: MessageRepo::new(&db),
            join_requests: JoinRequestRepo::new(&db),
            peer_reviews: PeerReviewRepo::new(&db),
            notifications: NotificationRepo::new(&db),
            competition_teams: CompetitionTeamRepo::new(&db),
            bookmarks: BookmarkRepo::new(&db),
            project_updates: ProjectUpdateRepo::new(&db),
            skill_catalog: SkillCatalogRepo::new(&db),
            chat: ChatState::new(),
            jwt_secret,
        }
    }
}
