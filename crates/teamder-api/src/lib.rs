#[macro_use]
extern crate rocket;

pub mod auth;
pub mod error;
pub mod guards;
pub mod routes;
pub mod state;

use rocket::fs::FileServer;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::str::FromStr;
use teamder_db::DbClient;
use state::AppState;

/// Builds the Rocket application with the given DB client and JWT secret.
/// Exposed for integration testing — production use goes through `main.rs`.
pub async fn build_rocket(db_client: DbClient, jwt_secret: String) -> rocket::Rocket<rocket::Build> {
    let app_state = AppState::new_with_secret(db_client, jwt_secret);

    // Ensure the uploads directory exists so the static FileServer can serve from it.
    let _ = std::fs::create_dir_all("uploads");

    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "http://localhost:3001",
        "https://teamder.watchandy.me",
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
        .mount("/uploads", FileServer::from("uploads").rank(10))
        .mount("/api/v1/auth", routes::auth::routes())
        .mount("/api/v1/users", routes::users::routes())
        .mount("/api/v1/projects", routes::projects::routes())
        .mount("/api/v1/competitions", routes::competitions::routes())
        .mount("/api/v1/study-groups", routes::study_groups::routes())
        .mount("/api/v1/invites", routes::invites::routes())
        .mount("/api/v1/admin", routes::admin::routes())
        .mount("/api/v1/uploads", routes::uploads::routes())
}

#[get("/")]
pub fn health_check() -> rocket::serde::json::Json<serde_json::Value> {
    rocket::serde::json::Json(serde_json::json!({
        "status": "ok",
        "service": "teamder-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
