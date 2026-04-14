/// One-off utility: drops all databases whose name starts with `teamder_test`
/// from the Atlas cluster configured in .env (MONGODB_URI).
///
/// Run with:
///   cargo run -p teamder-api --bin cleanup_test_dbs

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    println!("Connecting to MongoDB…");
    let client = mongodb::Client::with_uri_str(&uri)
        .await
        .expect("Failed to connect");

    let names = client
        .list_database_names()
        .await
        .expect("Failed to list databases");

    let test_dbs: Vec<_> = names
        .iter()
        .filter(|n| n.starts_with("teamder_test"))
        .collect();

    if test_dbs.is_empty() {
        println!("No teamder_test* databases found — nothing to do.");
        return;
    }

    println!("Found {} database(s) to drop:", test_dbs.len());
    for name in &test_dbs {
        println!("  - {name}");
    }

    for name in &test_dbs {
        client
            .database(name)
            .drop()
            .await
            .expect("Failed to drop database");
        println!("  ✓ Dropped {name}");
    }

    println!("Done — {} database(s) removed.", test_dbs.len());
}
