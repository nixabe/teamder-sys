//! End-to-end integration tests for the Teamder API.
//!
//! Tests use the `MONGODB_TEST_DB` env var as the test database name
//! (defaults to the value of `DB_NAME`, i.e. `teamder`).
//! All collections are wiped before each test and after the full run.
//!
//! ⚠️  WARNING: tests WILL destroy all data in the test database.
//!    After running tests, restart the server to re-seed the data.
//!
//! Run with:
//!   cargo test -p teamder-api --test api_tests -- --test-threads=1
//!
//! To use a dedicated test database (recommended if Atlas allows):
//!   MONGODB_TEST_DB=teamder_e2e cargo test -p teamder-api --test api_tests -- --test-threads=1

use mongodb::{Collection, bson::Document};
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};
use teamder_core::models::user::User;
use teamder_db::DbClient;

const TEST_SECRET: &str = "teamder-test-secret";

/// Ensures old `teamder_test*` databases are dropped exactly once per process.
static INITIAL_CLEANUP_DONE: AtomicBool = AtomicBool::new(false);

// ── Test Helpers ─────────────────────────────────────────────────────────────

/// Spin up a Rocket instance connected to the shared test database.
/// All collections are wiped at the start of every test for a clean slate.
/// Must be run with `-- --test-threads=1` to avoid inter-test interference.
async fn setup() -> (Client, DbClient) {
    // Load .env from this crate's directory or any parent (finds teamder-sys/.env)
    let _ = dotenvy::dotenv();

    let uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    // Use MONGODB_TEST_DB if set; otherwise fall back to DB_NAME (same DB as dev).
    // On Atlas free tier, creating NEW databases fails when the 100-DB limit is hit.
    // Using an existing DB avoids that limit.
    let db_name = std::env::var("MONGODB_TEST_DB")
        .or_else(|_| std::env::var("DB_NAME"))
        .unwrap_or_else(|_| "teamder".to_string());

    // One-time: drop old teamder_test_* databases from previous test runs to free Atlas slots
    if INITIAL_CLEANUP_DONE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        drop_old_test_databases(&uri).await;
    }

    let db = DbClient::connect(&uri, &db_name)
        .await
        .expect("Failed to connect to test MongoDB");

    // Wipe all collections so every test starts from a clean state
    wipe_collections(&db).await;

    let rocket = teamder_api::build_rocket(db.clone(), TEST_SECRET.to_string()).await;
    let client = Client::tracked(rocket)
        .await
        .expect("Failed to build Rocket test client");

    (client, db)
}

/// Drop all databases whose names start with `teamder_test` to free Atlas slots.
async fn drop_old_test_databases(uri: &str) {
    if let Ok(client) = mongodb::Client::with_uri_str(uri).await {
        if let Ok(names) = client.list_database_names().await {
            for name in names {
                if name.starts_with("teamder_test") {
                    client.database(&name).drop().await.ok();
                }
            }
        }
    }
}

/// Drop every collection in the test DB (not the DB itself, to stay within Atlas limits).
async fn wipe_collections(db: &DbClient) {
    let names = db.db.list_collection_names().await.unwrap_or_default();
    for name in names {
        db.db.collection::<Document>(&name).drop().await.ok();
    }
}

/// Clean up after a test — wipe collections for the next test.
async fn teardown(db: &DbClient) {
    wipe_collections(db).await;
}

/// Register a user via the API and return (token, user_id).
async fn register(client: &Client, email: &str, password: &str, name: &str) -> (String, String) {
    let resp = client
        .post("/api/v1/auth/register")
        .header(ContentType::JSON)
        .body(
            json!({
                "email": email,
                "password": password,
                "name": name,
                "role": "Developer",
                "department": "Computer Science"
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok, "register failed for {email}");
    let body: Value = resp.into_json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

/// Directly insert an admin user into the DB and return (user_id, token).
/// Used to test admin-only endpoints without going through a registration backdoor.
async fn create_admin(db: &DbClient) -> (String, String) {
    let hash = bcrypt::hash("admin1234", 4).expect("bcrypt failed");
    let mut admin = User::new("admin@test.com", hash, "Test Admin", "Admin", "CS");
    admin.is_admin = true;

    let col: Collection<User> = db.db.collection("users");
    col.insert_one(&admin).await.expect("insert admin failed");

    let token =
        teamder_api::auth::create_token(&admin.id, &admin.email, true, TEST_SECRET).unwrap();
    (admin.id, token)
}

/// Helper: build an Authorization header value.
fn bearer(token: &str) -> Header<'static> {
    Header::new("Authorization", format!("Bearer {token}"))
}

// ── Health Check ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_check() {
    let (client, db) = setup().await;

    let resp = client.get("/health").dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "teamder-api");

    teardown(&db).await;
}

// ── Auth: Register ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_register_success() {
    let (client, db) = setup().await;

    let resp = client
        .post("/api/v1/auth/register")
        .header(ContentType::JSON)
        .body(
            json!({
                "email": "alice@test.com",
                "password": "password123",
                "name": "Alice Wang",
                "role": "UI/UX Designer",
                "department": "Digital Media"
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "alice@test.com");
    assert_eq!(body["user"]["name"], "Alice Wang");
    assert_eq!(body["user"]["initials"], "AW");

    teardown(&db).await;
}

#[tokio::test]
async fn test_register_duplicate_email_returns_conflict() {
    let (client, db) = setup().await;

    let body = json!({
        "email": "dup@test.com",
        "password": "pass1",
        "name": "Dup User",
        "role": "Dev",
        "department": "CS"
    })
    .to_string();

    let r1 = client
        .post("/api/v1/auth/register")
        .header(ContentType::JSON)
        .body(body.clone())
        .dispatch()
        .await;
    assert_eq!(r1.status(), Status::Ok);

    let r2 = client
        .post("/api/v1/auth/register")
        .header(ContentType::JSON)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(r2.status(), Status::Conflict);

    teardown(&db).await;
}

#[tokio::test]
async fn test_register_missing_fields_returns_unprocessable() {
    let (client, db) = setup().await;

    // Missing required fields
    let resp = client
        .post("/api/v1/auth/register")
        .header(ContentType::JSON)
        .body(json!({ "email": "x@y.com" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::UnprocessableEntity);

    teardown(&db).await;
}

// ── Auth: Login ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_success() {
    let (client, db) = setup().await;
    register(&client, "bob@test.com", "secret123", "Bob Lin").await;

    let resp = client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(json!({ "email": "bob@test.com", "password": "secret123" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "bob@test.com");

    teardown(&db).await;
}

#[tokio::test]
async fn test_login_wrong_password_returns_unauthorized() {
    let (client, db) = setup().await;
    register(&client, "bob@test.com", "correct", "Bob Lin").await;

    let resp = client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(json!({ "email": "bob@test.com", "password": "wrong" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Unauthorized);

    teardown(&db).await;
}

#[tokio::test]
async fn test_login_unknown_email_returns_unauthorized() {
    let (client, db) = setup().await;

    let resp = client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(json!({ "email": "nobody@test.com", "password": "pass" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Unauthorized);

    teardown(&db).await;
}

// ── Users ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_users_returns_paginated_response() {
    let (client, db) = setup().await;
    register(&client, "u1@test.com", "p", "User One").await;
    register(&client, "u2@test.com", "p", "User Two").await;

    let resp = client.get("/api/v1/users").dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let body: Value = resp.into_json().await.unwrap();
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert!(body["meta"]["total"].as_u64().unwrap() >= 2);

    teardown(&db).await;
}

#[tokio::test]
async fn test_get_user_by_id_success() {
    let (client, db) = setup().await;
    let (_, user_id) = register(&client, "alice@test.com", "p", "Alice Wang").await;

    let resp = client
        .get(format!("/api/v1/users/{user_id}"))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["id"], user_id);
    assert_eq!(body["email"], "alice@test.com");

    teardown(&db).await;
}

#[tokio::test]
async fn test_get_user_not_found() {
    let (client, db) = setup().await;

    let resp = client
        .get("/api/v1/users/nonexistent-id-12345")
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::NotFound);

    teardown(&db).await;
}

#[tokio::test]
async fn test_me_without_auth_returns_unauthorized() {
    let (client, db) = setup().await;

    let resp = client.get("/api/v1/users/me").dispatch().await;
    assert_eq!(resp.status(), Status::Unauthorized);

    teardown(&db).await;
}

#[tokio::test]
async fn test_me_with_auth_returns_current_user() {
    let (client, db) = setup().await;
    let (token, user_id) = register(&client, "me@test.com", "pass", "Me User").await;

    let resp = client
        .get("/api/v1/users/me")
        .header(bearer(&token))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["id"], user_id);

    teardown(&db).await;
}

#[tokio::test]
async fn test_update_own_profile_success() {
    let (client, db) = setup().await;
    let (token, user_id) = register(&client, "u@test.com", "p", "Old Name").await;

    let resp = client
        .patch(format!("/api/v1/users/{user_id}"))
        .header(ContentType::JSON)
        .header(bearer(&token))
        .body(json!({ "location": "Taipei, Taiwan" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["success"], true);

    teardown(&db).await;
}

#[tokio::test]
async fn test_update_other_profile_returns_forbidden() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "User A").await;
    let (_, id_b) = register(&client, "b@test.com", "p", "User B").await;

    let resp = client
        .patch(format!("/api/v1/users/{id_b}"))
        .header(ContentType::JSON)
        .header(bearer(&token_a))
        .body(json!({ "location": "Malicious update" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

#[tokio::test]
async fn test_delete_own_account_success() {
    let (client, db) = setup().await;
    let (token, user_id) = register(&client, "delete@test.com", "p", "Delete Me").await;

    let resp = client
        .delete(format!("/api/v1/users/{user_id}"))
        .header(bearer(&token))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);

    // Confirm user is gone
    let check = client
        .get(format!("/api/v1/users/{user_id}"))
        .dispatch()
        .await;
    assert_eq!(check.status(), Status::NotFound);

    teardown(&db).await;
}

#[tokio::test]
async fn test_delete_other_account_returns_forbidden() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "User A").await;
    let (_, id_b) = register(&client, "b@test.com", "p", "User B").await;

    let resp = client
        .delete(format!("/api/v1/users/{id_b}"))
        .header(bearer(&token_a))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

#[tokio::test]
async fn test_search_users_by_query() {
    let (client, db) = setup().await;
    register(&client, "rust@test.com", "p", "Rustacean Dev").await;
    register(&client, "go@test.com", "p", "Gopher Dev").await;

    let resp = client
        .get("/api/v1/users?q=Rustacean")
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "Rustacean Dev");

    teardown(&db).await;
}

// ── Projects ──────────────────────────────────────────────────────────────────

async fn create_project(client: &Client, token: &str, name: &str) -> String {
    let resp = client
        .post("/api/v1/projects")
        .header(ContentType::JSON)
        .header(bearer(token))
        .body(
            json!({
                "name": name,
                "description": "A test project description",
                "skills": ["Rust", "TypeScript"],
                "collab": "hybrid"
            })
            .to_string(),
        )
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok, "create_project failed");
    let body: Value = resp.into_json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_list_projects_returns_paginated_response() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "User").await;
    create_project(&client, &token, "Project A").await;
    create_project(&client, &token, "Project B").await;

    let resp = client.get("/api/v1/projects").dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let body: Value = resp.into_json().await.unwrap();
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 2);

    teardown(&db).await;
}

#[tokio::test]
async fn test_create_project_without_auth_returns_unauthorized() {
    let (client, db) = setup().await;

    let resp = client
        .post("/api/v1/projects")
        .header(ContentType::JSON)
        .body(
            json!({
                "name": "Unauthorized Project",
                "description": "Should fail",
                "skills": [],
                "collab": "remote"
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Unauthorized);

    teardown(&db).await;
}

#[tokio::test]
async fn test_create_project_with_auth_success() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "Alice Wang").await;

    let resp = client
        .post("/api/v1/projects")
        .header(ContentType::JSON)
        .header(bearer(&token))
        .body(
            json!({
                "name": "My New Project",
                "description": "A detailed description of the project",
                "skills": ["Rust", "MongoDB"],
                "collab": "remote",
                "category": "Backend"
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["name"], "My New Project");
    assert_eq!(body["status"], "recruiting");

    teardown(&db).await;
}

#[tokio::test]
async fn test_get_project_by_id_success() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "User").await;
    let project_id = create_project(&client, &token, "Findable Project").await;

    let resp = client
        .get(format!("/api/v1/projects/{project_id}"))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["id"], project_id);
    assert_eq!(body["name"], "Findable Project");

    teardown(&db).await;
}

#[tokio::test]
async fn test_get_project_not_found() {
    let (client, db) = setup().await;

    let resp = client
        .get("/api/v1/projects/does-not-exist")
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::NotFound);

    teardown(&db).await;
}

#[tokio::test]
async fn test_my_projects_returns_only_own_projects() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, _) = register(&client, "b@test.com", "p", "Bob").await;

    create_project(&client, &token_a, "Alice's Project").await;
    create_project(&client, &token_b, "Bob's Project").await;

    let resp = client
        .get("/api/v1/projects/my")
        .header(bearer(&token_a))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "Alice's Project");

    teardown(&db).await;
}

#[tokio::test]
async fn test_update_project_as_owner_success() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "User").await;
    let project_id = create_project(&client, &token, "Old Name").await;

    let resp = client
        .patch(format!("/api/v1/projects/{project_id}"))
        .header(ContentType::JSON)
        .header(bearer(&token))
        .body(json!({ "name": "Updated Name", "status": "active" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["success"], true);

    teardown(&db).await;
}

#[tokio::test]
async fn test_update_project_as_non_owner_returns_forbidden() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, _) = register(&client, "b@test.com", "p", "Bob").await;
    let project_id = create_project(&client, &token_a, "Alice's Project").await;

    let resp = client
        .patch(format!("/api/v1/projects/{project_id}"))
        .header(ContentType::JSON)
        .header(bearer(&token_b))
        .body(json!({ "name": "Bob Hijacked" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

#[tokio::test]
async fn test_delete_project_as_owner_success() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "User").await;
    let project_id = create_project(&client, &token, "Doomed Project").await;

    let resp = client
        .delete(format!("/api/v1/projects/{project_id}"))
        .header(bearer(&token))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);

    let check = client
        .get(format!("/api/v1/projects/{project_id}"))
        .dispatch()
        .await;
    assert_eq!(check.status(), Status::NotFound);

    teardown(&db).await;
}

#[tokio::test]
async fn test_delete_project_as_non_owner_returns_forbidden() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, _) = register(&client, "b@test.com", "p", "Bob").await;
    let project_id = create_project(&client, &token_a, "Alice's Project").await;

    let resp = client
        .delete(format!("/api/v1/projects/{project_id}"))
        .header(bearer(&token_b))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

#[tokio::test]
async fn test_list_projects_filter_by_status() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "User").await;
    let project_id = create_project(&client, &token, "Active Project").await;

    // Update it to active status
    client
        .patch(format!("/api/v1/projects/{project_id}"))
        .header(ContentType::JSON)
        .header(bearer(&token))
        .body(json!({ "status": "active" }).to_string())
        .dispatch()
        .await;

    create_project(&client, &token, "Recruiting Project").await;

    let resp = client
        .get("/api/v1/projects?status=active")
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["status"], "active");

    teardown(&db).await;
}

// ── Competitions ──────────────────────────────────────────────────────────────

async fn create_competition_as_admin(client: &Client, admin_token: &str, name: &str) -> String {
    let resp = client
        .post("/api/v1/competitions")
        .header(ContentType::JSON)
        .header(bearer(admin_token))
        .body(
            json!({
                "name": name,
                "organizer": "Test Org",
                "description": "A test competition",
                "prize": "NT$10,000",
                "team_size_min": 2,
                "team_size_max": 4,
                "duration": "48 hours",
                "tags": ["Hackathon"]
            })
            .to_string(),
        )
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok, "create_competition failed");
    let body: Value = resp.into_json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_list_competitions_returns_data() {
    let (client, db) = setup().await;
    let (_, admin_token) = create_admin(&db).await;
    create_competition_as_admin(&client, &admin_token, "Hackathon 2026").await;

    let resp = client.get("/api/v1/competitions").dispatch().await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    teardown(&db).await;
}

#[tokio::test]
async fn test_list_featured_competitions() {
    let (client, db) = setup().await;
    let (_, admin_token) = create_admin(&db).await;

    // Create featured
    client
        .post("/api/v1/competitions")
        .header(ContentType::JSON)
        .header(bearer(&admin_token))
        .body(
            json!({
                "name": "Featured Hack",
                "organizer": "Org",
                "description": "desc",
                "prize": "NT$5000",
                "team_size_min": 1,
                "team_size_max": 3,
                "duration": "24h",
                "tags": [],
                "is_featured": true
            })
            .to_string(),
        )
        .dispatch()
        .await;

    // Create non-featured
    create_competition_as_admin(&client, &admin_token, "Normal Hack").await;

    let resp = client
        .get("/api/v1/competitions/featured")
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "Featured Hack");

    teardown(&db).await;
}

#[tokio::test]
async fn test_get_competition_not_found() {
    let (client, db) = setup().await;

    let resp = client
        .get("/api/v1/competitions/no-such-id")
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NotFound);

    teardown(&db).await;
}

#[tokio::test]
async fn test_create_competition_non_admin_returns_forbidden() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "user@test.com", "p", "Regular User").await;

    let resp = client
        .post("/api/v1/competitions")
        .header(ContentType::JSON)
        .header(bearer(&token))
        .body(
            json!({
                "name": "Sneaky Competition",
                "organizer": "Me",
                "description": "desc",
                "prize": "lots",
                "team_size_min": 1,
                "team_size_max": 2,
                "duration": "1h",
                "tags": []
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

#[tokio::test]
async fn test_register_for_competition_success() {
    let (client, db) = setup().await;
    let (_, admin_token) = create_admin(&db).await;
    let (user_token, _) = register(&client, "contestant@test.com", "p", "Contestant").await;
    let comp_id = create_competition_as_admin(&client, &admin_token, "Open Hack").await;

    let resp = client
        .post(format!("/api/v1/competitions/{comp_id}/register"))
        .header(ContentType::JSON)
        .header(bearer(&user_token))
        .body(json!({ "team_name": "Rust Rockets" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["success"], true);

    teardown(&db).await;
}

#[tokio::test]
async fn test_register_for_competition_without_auth_returns_unauthorized() {
    let (client, db) = setup().await;
    let (_, admin_token) = create_admin(&db).await;
    let comp_id = create_competition_as_admin(&client, &admin_token, "Hack").await;

    let resp = client
        .post(format!("/api/v1/competitions/{comp_id}/register"))
        .header(ContentType::JSON)
        .body(json!({}).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Unauthorized);

    teardown(&db).await;
}

// ── Study Groups ──────────────────────────────────────────────────────────────

async fn create_study_group(client: &Client, token: &str, name: &str) -> String {
    let resp = client
        .post("/api/v1/study-groups")
        .header(ContentType::JSON)
        .header(bearer(token))
        .body(
            json!({
                "name": name,
                "goal": "Learn together",
                "subject": "Computer Science",
                "tags": ["Rust"],
                "schedule": "Saturdays 14:00",
                "duration_weeks": 8
            })
            .to_string(),
        )
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok, "create_study_group failed");
    let body: Value = resp.into_json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_list_study_groups_returns_paginated_response() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "User").await;
    create_study_group(&client, &token, "Rust Study").await;
    create_study_group(&client, &token, "Go Study").await;

    let resp = client.get("/api/v1/study-groups").dispatch().await;
    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);

    teardown(&db).await;
}

#[tokio::test]
async fn test_create_study_group_without_auth_returns_unauthorized() {
    let (client, db) = setup().await;

    let resp = client
        .post("/api/v1/study-groups")
        .header(ContentType::JSON)
        .body(
            json!({
                "name": "Ghost Group",
                "goal": "...",
                "subject": "CS",
                "tags": [],
                "schedule": "Never",
                "duration_weeks": 1
            })
            .to_string(),
        )
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Unauthorized);

    teardown(&db).await;
}

#[tokio::test]
async fn test_get_study_group_not_found() {
    let (client, db) = setup().await;

    let resp = client
        .get("/api/v1/study-groups/no-such-id")
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::NotFound);

    teardown(&db).await;
}

#[tokio::test]
async fn test_join_study_group_success() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, _) = register(&client, "b@test.com", "p", "Bob Lin").await;
    let group_id = create_study_group(&client, &token_a, "Open Group").await;

    let resp = client
        .post(format!("/api/v1/study-groups/{group_id}/join"))
        .header(bearer(&token_b))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify member_count increased
    let group_resp: Value = client
        .get(format!("/api/v1/study-groups/{group_id}"))
        .dispatch()
        .await
        .into_json()
        .await
        .unwrap();
    assert_eq!(group_resp["member_count"], 1);

    teardown(&db).await;
}

#[tokio::test]
async fn test_join_closed_study_group_returns_conflict() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, _) = register(&client, "b@test.com", "p", "Bob").await;

    // Create a group with max 1 member, then fill it with creator joining
    let resp = client
        .post("/api/v1/study-groups")
        .header(ContentType::JSON)
        .header(bearer(&token_a))
        .body(
            json!({
                "name": "Tiny Group",
                "goal": "Solo study",
                "subject": "CS",
                "tags": [],
                "schedule": "Never",
                "duration_weeks": 4,
                "max_members": 1
            })
            .to_string(),
        )
        .dispatch()
        .await;
    let body: Value = resp.into_json().await.unwrap();
    let group_id = body["id"].as_str().unwrap().to_string();

    // Fill it (token_a joins, making it full at 1 member)
    client
        .post(format!("/api/v1/study-groups/{group_id}/join"))
        .header(bearer(&token_a))
        .dispatch()
        .await;

    // Bob tries to join but it's full
    let resp = client
        .post(format!("/api/v1/study-groups/{group_id}/join"))
        .header(bearer(&token_b))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Conflict);

    teardown(&db).await;
}

#[tokio::test]
async fn test_checkin_to_study_group_success() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let group_id = create_study_group(&client, &token_a, "Study Group").await;

    // Join first
    client
        .post(format!("/api/v1/study-groups/{group_id}/join"))
        .header(bearer(&token_a))
        .dispatch()
        .await;

    let resp = client
        .post(format!("/api/v1/study-groups/{group_id}/checkin"))
        .header(bearer(&token_a))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["success"], true);

    teardown(&db).await;
}

// ── Invites ───────────────────────────────────────────────────────────────────

async fn send_invite(
    client: &Client,
    sender_token: &str,
    to_user_id: &str,
) -> String {
    let resp = client
        .post("/api/v1/invites")
        .header(ContentType::JSON)
        .header(bearer(sender_token))
        .body(
            json!({
                "to_user_id": to_user_id,
                "message": "Join my team!"
            })
            .to_string(),
        )
        .dispatch()
        .await;
    assert_eq!(resp.status(), Status::Ok, "send_invite failed");
    let body: Value = resp.into_json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_send_invite_success() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (_, id_b) = register(&client, "b@test.com", "p", "Bob").await;

    let resp = client
        .post("/api/v1/invites")
        .header(ContentType::JSON)
        .header(bearer(&token_a))
        .body(json!({ "to_user_id": id_b, "message": "Join me!" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["to_user_id"], id_b);
    assert_eq!(body["status"], "pending");

    teardown(&db).await;
}

#[tokio::test]
async fn test_send_invite_to_nonexistent_user_returns_not_found() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;

    let resp = client
        .post("/api/v1/invites")
        .header(ContentType::JSON)
        .header(bearer(&token_a))
        .body(json!({ "to_user_id": "ghost-user-id" }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::NotFound);

    teardown(&db).await;
}

#[tokio::test]
async fn test_list_invites_for_current_user() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, id_b) = register(&client, "b@test.com", "p", "Bob").await;

    send_invite(&client, &token_a, &id_b).await;

    let resp = client
        .get("/api/v1/invites")
        .header(bearer(&token_b))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["status"], "pending");

    teardown(&db).await;
}

#[tokio::test]
async fn test_get_invite_as_non_participant_returns_forbidden() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (_, id_b) = register(&client, "b@test.com", "p", "Bob").await;
    let (token_c, _) = register(&client, "c@test.com", "p", "Carol").await;

    let invite_id = send_invite(&client, &token_a, &id_b).await;

    let resp = client
        .get(format!("/api/v1/invites/{invite_id}"))
        .header(bearer(&token_c))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

#[tokio::test]
async fn test_respond_invite_accept_success() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, id_b) = register(&client, "b@test.com", "p", "Bob").await;
    let invite_id = send_invite(&client, &token_a, &id_b).await;

    let resp = client
        .post(format!("/api/v1/invites/{invite_id}/respond"))
        .header(ContentType::JSON)
        .header(bearer(&token_b))
        .body(json!({ "accept": true }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["status"], "accepted");

    teardown(&db).await;
}

#[tokio::test]
async fn test_respond_invite_decline_success() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, id_b) = register(&client, "b@test.com", "p", "Bob").await;
    let invite_id = send_invite(&client, &token_a, &id_b).await;

    let resp = client
        .post(format!("/api/v1/invites/{invite_id}/respond"))
        .header(ContentType::JSON)
        .header(bearer(&token_b))
        .body(json!({ "accept": false }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["status"], "declined");

    teardown(&db).await;
}

#[tokio::test]
async fn test_respond_to_non_pending_invite_returns_conflict() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (token_b, id_b) = register(&client, "b@test.com", "p", "Bob").await;
    let invite_id = send_invite(&client, &token_a, &id_b).await;

    // Accept once
    client
        .post(format!("/api/v1/invites/{invite_id}/respond"))
        .header(ContentType::JSON)
        .header(bearer(&token_b))
        .body(json!({ "accept": true }).to_string())
        .dispatch()
        .await;

    // Accept again → Conflict
    let resp = client
        .post(format!("/api/v1/invites/{invite_id}/respond"))
        .header(ContentType::JSON)
        .header(bearer(&token_b))
        .body(json!({ "accept": true }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Conflict);

    teardown(&db).await;
}

#[tokio::test]
async fn test_sender_cannot_respond_to_own_invite() {
    let (client, db) = setup().await;
    let (token_a, _) = register(&client, "a@test.com", "p", "Alice").await;
    let (_, id_b) = register(&client, "b@test.com", "p", "Bob").await;
    let invite_id = send_invite(&client, &token_a, &id_b).await;

    // Sender tries to accept their own invite
    let resp = client
        .post(format!("/api/v1/invites/{invite_id}/respond"))
        .header(ContentType::JSON)
        .header(bearer(&token_a))
        .body(json!({ "accept": true }).to_string())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

// ── Admin ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_stats_non_admin_returns_forbidden() {
    let (client, db) = setup().await;
    let (token, _) = register(&client, "u@test.com", "p", "Regular User").await;

    let resp = client
        .get("/api/v1/admin/stats")
        .header(bearer(&token))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Forbidden);

    teardown(&db).await;
}

#[tokio::test]
async fn test_admin_stats_without_auth_returns_unauthorized() {
    let (client, db) = setup().await;

    let resp = client.get("/api/v1/admin/stats").dispatch().await;
    assert_eq!(resp.status(), Status::Unauthorized);

    teardown(&db).await;
}

#[tokio::test]
async fn test_admin_stats_success() {
    let (client, db) = setup().await;
    let (_, admin_token) = create_admin(&db).await;
    register(&client, "u@test.com", "p", "User").await;

    let resp = client
        .get("/api/v1/admin/stats")
        .header(bearer(&admin_token))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert!(body["users"].as_u64().unwrap() >= 1);
    assert!(body["projects"].is_number());
    assert!(body["competitions"].is_number());
    assert!(body["study_groups"].is_number());

    teardown(&db).await;
}

#[tokio::test]
async fn test_admin_list_users_success() {
    let (client, db) = setup().await;
    let (_, admin_token) = create_admin(&db).await;
    register(&client, "u1@test.com", "p", "User 1").await;
    register(&client, "u2@test.com", "p", "User 2").await;

    let resp = client
        .get("/api/v1/admin/users")
        .header(bearer(&admin_token))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    // Admin user + 2 registered = 3 total
    assert!(body["meta"]["total"].as_u64().unwrap() >= 3);

    teardown(&db).await;
}

#[tokio::test]
async fn test_admin_list_projects_success() {
    let (client, db) = setup().await;
    let (_, admin_token) = create_admin(&db).await;
    let (user_token, _) = register(&client, "u@test.com", "p", "User").await;
    create_project(&client, &user_token, "Admin Visible Project").await;

    let resp = client
        .get("/api/v1/admin/projects")
        .header(bearer(&admin_token))
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let body: Value = resp.into_json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    teardown(&db).await;
}
