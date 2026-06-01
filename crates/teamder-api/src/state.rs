use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use teamder_db::{
    DbClient,
    repos::{
        AuthCodeRepo, BookmarkRepo, CompetitionRepo, CompetitionTeamRepo, ContactExchangeRepo,
        InviteRepo, JoinRequestRepo, MessageRepo, NotificationRepo, PeerReviewRepo, ProjectRepo,
        ProjectUpdateRepo, ReportRepo, SkillCatalogRepo, StudyGroupRepo,
        StudyGroupAnnouncementRepo, StudyGroupEventRepo, UserRepo,
    },
};

use crate::{chat::ChatState, mailer::Mailer};

/// Shared application state injected into every Rocket handler.
pub struct AppState {
    pub users: UserRepo,
    pub auth_codes: AuthCodeRepo,
    pub mailer: Mailer,
    pub projects: ProjectRepo,
    pub competitions: CompetitionRepo,
    pub study_groups: StudyGroupRepo,
    pub invites: InviteRepo,
    pub messages: MessageRepo,
    pub join_requests: JoinRequestRepo,
    pub contact_exchanges: ContactExchangeRepo,
    pub sg_announcements: StudyGroupAnnouncementRepo,
    pub sg_events: StudyGroupEventRepo,
    pub peer_reviews: PeerReviewRepo,
    pub notifications: NotificationRepo,
    pub competition_teams: CompetitionTeamRepo,
    pub bookmarks: BookmarkRepo,
    pub reports: ReportRepo,
    pub project_updates: ProjectUpdateRepo,
    pub skill_catalog: SkillCatalogRepo,
    pub chat: ChatState,
    pub notif_hub: ChatState,
    pub jwt_secret: String,
    /// Tracks when a user last left a project. Key = "{user_id}:{project_id}"
    pub leave_log: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
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
            auth_codes: AuthCodeRepo::new(&db),
            mailer: Mailer::from_env(),
            projects: ProjectRepo::new(&db),
            competitions: CompetitionRepo::new(&db),
            study_groups: StudyGroupRepo::new(&db),
            invites: InviteRepo::new(&db),
            messages: MessageRepo::new(&db),
            join_requests: JoinRequestRepo::new(&db),
            contact_exchanges: ContactExchangeRepo::new(&db),
            sg_announcements: StudyGroupAnnouncementRepo::new(&db),
            sg_events: StudyGroupEventRepo::new(&db),
            peer_reviews: PeerReviewRepo::new(&db),
            notifications: NotificationRepo::new(&db),
            competition_teams: CompetitionTeamRepo::new(&db),
            bookmarks: BookmarkRepo::new(&db),
            reports: ReportRepo::new(&db),
            project_updates: ProjectUpdateRepo::new(&db),
            skill_catalog: SkillCatalogRepo::new(&db),
            chat: ChatState::new(),
            notif_hub: ChatState::new(),
            jwt_secret,
            leave_log: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
