use teamder_db::{
    DbClient,
    repos::{
        CompetitionRepo, InviteRepo, JoinRequestRepo, MessageRepo, NotificationRepo,
        PeerReviewRepo, ProjectRepo, StudyGroupRepo, UserRepo,
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
            chat: ChatState::new(),
            jwt_secret,
        }
    }
}
