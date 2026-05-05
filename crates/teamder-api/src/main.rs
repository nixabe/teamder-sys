use rocket::fairing::AdHoc;
use teamder_db::{seed, DbClient};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
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
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "teamder-dev-secret-change-in-production".to_string());

    // Connect to MongoDB
    let db_client = DbClient::connect(&mongo_uri, &db_name)
        .await
        .expect("Failed to connect to MongoDB");

    seed::seed_if_empty(&db_client)
        .await
        .expect("Failed to seed database");
    // Catalog seed runs independently so existing deployments get the
    // skill_categories collection populated on first boot of this version.
    seed::seed_skill_catalog_if_empty(&db_client)
        .await
        .expect("Failed to seed skill catalog");

    teamder_api::build_rocket(db_client, jwt_secret)
        .await
        .attach(AdHoc::on_liftoff("Server ready", |_| {
            Box::pin(async {
                tracing::info!("🚀 Teamder API is live");
            })
        }))
        .launch()
        .await?;

    Ok(())
}
