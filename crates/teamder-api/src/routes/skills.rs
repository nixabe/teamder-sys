use rocket::{Route, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::skills::catalog;

/// GET /api/v1/skills — return the hardcoded skill catalog grouped by category.
#[get("/")]
fn get_catalog() -> Json<Value> {
    let cats = catalog();
    Json(json!({ "categories": cats }))
}

pub fn routes() -> Vec<Route> {
    routes![get_catalog]
}
