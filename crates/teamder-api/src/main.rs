use rocket::fs::FileServer;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use tracing_subscriber::EnvFilter;

use teamder_api::routes;
use teamder_api::state::{AppState, ChatState};
use teamder_db::client::DbClient;
use teamder_db::seed::seed_if_empty;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file (silently ignore if missing).
    let _ = dotenvy::dotenv();

    // Initialise structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Read configuration from environment.
    let mongo_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "teamder".to_string());
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "teamder_dev_secret_change_me".to_string());

    // Connect to MongoDB.
    let db = DbClient::new(&mongo_uri, &db_name).await?;

    // Seed initial data if collections are empty.
    seed_if_empty(&db).await?;

    // Configure CORS.
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "http://localhost:3001",
        "https://teamder.watchandy.me",
    ]);

    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Patch,
            Method::Delete,
            Method::Options,
        ]
        .into_iter()
        .map(From::from)
        .collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Content-Type",
            "Accept",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    let app_state = AppState {
        db,
        jwt_secret,
        chat_state: ChatState::new(),
    };

    // Build and launch Rocket.
    rocket::build()
        .manage(app_state)
        // Health
        .mount("/api/v1", rocket::routes![routes::health::health_check])
        // Auth
        .mount(
            "/api/v1",
            rocket::routes![
                routes::auth::register,
                routes::auth::login,
                routes::auth::forgot_password,
                routes::auth::reset_password,
            ],
        )
        // Users
        .mount(
            "/api/v1",
            rocket::routes![
                routes::users::list_users,
                routes::users::get_me,
                routes::users::get_user,
                routes::users::update_user,
                routes::users::delete_user,
                routes::users::change_password,
                routes::users::onboard,
            ],
        )
        // Uploads
        .mount(
            "/api/v1",
            rocket::routes![
                routes::uploads::upload_avatar,
                routes::uploads::upload_portfolio,
                routes::uploads::upload_resume,
                routes::uploads::upload_banner,
                routes::uploads::upload_application,
                routes::uploads::delete_upload,
            ],
        )
        // Projects
        .mount(
            "/api/v1",
            rocket::routes![
                routes::projects::list_projects,
                routes::projects::my_projects,
                routes::projects::joined_projects,
                routes::projects::get_project,
                routes::projects::create_project,
                routes::projects::update_project,
                routes::projects::delete_project,
                routes::projects::recommend_users,
                routes::projects::complete_project,
                routes::projects::leave_project,
                routes::projects::remove_member,
                routes::projects::set_role,
            ],
        )
        // Project Updates
        .mount(
            "/api/v1",
            rocket::routes![
                routes::project_updates::list_updates,
                routes::project_updates::create_update,
                routes::project_updates::delete_update,
            ],
        )
        // Competitions
        .mount(
            "/api/v1",
            rocket::routes![
                routes::competitions::list_competitions,
                routes::competitions::featured_competitions,
                routes::competitions::my_competitions,
                routes::competitions::pending_competitions,
                routes::competitions::get_competition,
                routes::competitions::create_competition,
                routes::competitions::update_competition,
                routes::competitions::register_competition,
                routes::competitions::toggle_interest,
                routes::competitions::get_registrations,
                routes::competitions::submit_review,
                routes::competitions::approve_competition,
                routes::competitions::reject_competition,
                routes::competitions::set_winners,
            ],
        )
        // Competition Teams
        .mount(
            "/api/v1",
            rocket::routes![
                routes::competition_teams::create_team,
                routes::competition_teams::get_team,
                routes::competition_teams::update_team,
                routes::competition_teams::apply_to_team,
                routes::competition_teams::accept_member,
                routes::competition_teams::list_applications,
                routes::competition_teams::leave_team,
                routes::competition_teams::teams_by_competition,
                routes::competition_teams::my_teams,
            ],
        )
        // Study Groups
        .mount(
            "/api/v1",
            rocket::routes![
                routes::study_groups::list_study_groups,
                routes::study_groups::joined_study_groups,
                routes::study_groups::get_study_group,
                routes::study_groups::create_study_group,
                routes::study_groups::join_study_group,
                routes::study_groups::checkin,
                routes::study_groups::add_note,
                routes::study_groups::delete_note,
                routes::study_groups::leave_study_group,
                routes::study_groups::set_progress,
                routes::study_groups::complete_study_group,
                routes::study_groups::update_study_group,
                routes::study_groups::delete_study_group,
            ],
        )
        // Invites
        .mount(
            "/api/v1",
            rocket::routes![
                routes::invites::list_invites,
                routes::invites::get_invite,
                routes::invites::send_invite,
                routes::invites::respond_invite,
                routes::invites::mark_read,
                routes::invites::read_all,
                routes::invites::delete_invite,
            ],
        )
        // Join Requests
        .mount(
            "/api/v1",
            rocket::routes![
                routes::join_requests::create_join_request,
                routes::join_requests::incoming_requests,
                routes::join_requests::sent_requests,
                routes::join_requests::respond_request,
            ],
        )
        // Peer Reviews
        .mount(
            "/api/v1",
            rocket::routes![
                routes::peer_reviews::reviews_for_user,
                routes::peer_reviews::create_review,
            ],
        )
        // Chat
        .mount(
            "/api/v1",
            rocket::routes![
                routes::chat::list_conversations,
                routes::chat::get_messages,
                routes::chat::send_message,
                routes::chat::websocket_handler,
            ],
        )
        // Notifications
        .mount(
            "/api/v1",
            rocket::routes![
                routes::notifications::list_notifications,
                routes::notifications::mark_read,
                routes::notifications::read_all,
            ],
        )
        // Bookmarks
        .mount(
            "/api/v1",
            rocket::routes![
                routes::bookmarks::list_bookmarks,
                routes::bookmarks::add_bookmark,
                routes::bookmarks::remove_bookmark,
            ],
        )
        // Search
        .mount(
            "/api/v1",
            rocket::routes![routes::search::search],
        )
        // Skills
        .mount(
            "/api/v1",
            rocket::routes![routes::skills::get_catalog],
        )
        // Admin
        .mount(
            "/api/v1",
            rocket::routes![
                routes::admin::stats,
                routes::admin::timeseries,
                routes::admin::list_users,
                routes::admin::list_projects,
                routes::admin::promote_user,
                routes::admin::promote_project,
                routes::admin::toggle_publisher,
                routes::admin::delete_user,
                routes::admin::delete_project,
                routes::admin::export_users_csv,
                routes::admin::list_study_groups,
                routes::admin::list_competitions,
            ],
        )
        // Admin Skills
        .mount(
            "/api/v1",
            rocket::routes![
                routes::admin_skills::list_skills,
                routes::admin_skills::create_category,
                routes::admin_skills::update_category,
                routes::admin_skills::delete_category,
                routes::admin_skills::create_tag,
                routes::admin_skills::update_tag,
                routes::admin_skills::delete_tag,
            ],
        )
        // Static file serving for uploads
        .mount("/uploads", FileServer::from("uploads"))
        .attach(cors)
        .launch()
        .await?;

    Ok(())
}
