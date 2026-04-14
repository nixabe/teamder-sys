use teamder_db::{
    DbClient,
    repos::{
        CompetitionRepo, InviteRepo, ProjectRepo, StudyGroupRepo, UserRepo,
    },
};

/// Shared application state injected into every Rocket handler.
pub struct AppState {
    pub users: UserRepo,
    pub projects: ProjectRepo,
    pub competitions: CompetitionRepo,
    pub study_groups: StudyGroupRepo,
    pub invites: InviteRepo,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db: DbClient) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "teamder-dev-secret-change-in-production".to_string());
        Self {
            users: UserRepo::new(&db),
            projects: ProjectRepo::new(&db),
            competitions: CompetitionRepo::new(&db),
            study_groups: StudyGroupRepo::new(&db),
            invites: InviteRepo::new(&db),
            jwt_secret,
        }
    }
}
