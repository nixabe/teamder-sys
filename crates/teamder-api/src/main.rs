#[macro_use]
extern crate rocket;

mod auth;
mod error;
mod guards;
mod routes;
mod state;

use rocket::fairing::AdHoc;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::str::FromStr;
use teamder_db::{seed, DbClient};

use crate::state::AppState;

#[launch]
async fn rocket() -> _ {
    // Load .env if present
    let _ = dotenvy::dotenv();

    // Tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Config from environment
    let mongo_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "teamder".to_string());

    // Connect to MongoDB
    let db_client = DbClient::connect(&mongo_uri, &db_name)
        .await
        .expect("Failed to connect to MongoDB");

    seed::seed_if_empty(&db_client)
        .await
        .expect("Failed to seed database");

    let app_state = AppState::new(db_client);

    // CORS setup — allow the Next.js dev origin by default
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "http://localhost:3001",
        "https://teamder.app",
    ]);
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]
            .into_iter()
            .map(|s| FromStr::from_str(s).unwrap())
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("CORS configuration error");

    rocket::build()
        .manage(app_state)
        .attach(cors)
        .mount("/health", routes![health_check])
        .mount("/api/v1/auth", routes::auth::routes())
        .mount("/api/v1/users", routes::users::routes())
        .mount("/api/v1/projects", routes::projects::routes())
        .mount("/api/v1/competitions", routes::competitions::routes())
        .mount("/api/v1/study-groups", routes::study_groups::routes())
        .mount("/api/v1/invites", routes::invites::routes())
        .mount("/api/v1/admin", routes::admin::routes())
        .attach(AdHoc::on_liftoff("Server ready", |_| {
            Box::pin(async {
                tracing::info!("🚀 Teamder API is live");
            })
        }))
}

#[get("/")]
fn health_check() -> rocket::serde::json::Json<serde_json::Value> {
    rocket::serde::json::Json(serde_json::json!({
        "status": "ok",
        "service": "teamder-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
