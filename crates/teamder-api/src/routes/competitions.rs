use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::competition::{
        Competition, CompetitionResponse, CreateCompetitionRequest,
        RegisterCompetitionRequest, Registration,
    },
};
use chrono::Utc;

use crate::{error::ApiResult, guards::{AdminUser, AuthUser}, state::AppState};

/// GET /api/v1/competitions
#[get("/")]
async fn list_competitions(state: &State<AppState>) -> ApiResult<Value> {
    let comps: Vec<CompetitionResponse> = state
        .competitions
        .list()
        .await?
        .into_iter()
        .map(CompetitionResponse::from)
        .collect();

    Ok(Json(json!({ "data": comps })))
}

/// GET /api/v1/competitions/featured
#[get("/featured")]
async fn featured_competitions(state: &State<AppState>) -> ApiResult<Value> {
    let comps: Vec<CompetitionResponse> = state
        .competitions
        .list_featured()
        .await?
        .into_iter()
        .map(CompetitionResponse::from)
        .collect();

    Ok(Json(json!({ "data": comps })))
}

/// GET /api/v1/competitions/<id>
#[get("/<id>")]
async fn get_competition(id: String, state: &State<AppState>) -> ApiResult<CompetitionResponse> {
    let comp = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;
    Ok(Json(comp.into()))
}

/// POST /api/v1/competitions  (admin only)
#[post("/", data = "<req>")]
async fn create_competition(
    req: Json<CreateCompetitionRequest>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<CompetitionResponse> {
    let mut comp = Competition::new(&req.name, &req.organizer, &req.description);
    comp.prize = req.prize.clone();
    comp.team_size_min = req.team_size_min;
    comp.team_size_max = req.team_size_max;
    comp.deadline = req.deadline.clone();
    comp.duration = req.duration.clone();
    comp.tags = req.tags.clone();
    comp.is_featured = req.is_featured.unwrap_or(false);
    if let Some(v) = &req.icon { comp.icon = v.clone(); }
    if let Some(v) = &req.icon_bg { comp.icon_bg = v.clone(); }

    state.competitions.create(&comp).await?;
    Ok(Json(comp.into()))
}

/// POST /api/v1/competitions/<id>/register  (auth)
#[post("/<id>/register", data = "<req>")]
async fn register_competition(
    id: String,
    req: Json<RegisterCompetitionRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    // Verify competition exists
    let _ = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;

    let registration = Registration {
        user_id: auth.0.sub.clone(),
        team_name: req.team_name.clone(),
        registered_at: Utc::now(),
    };

    state.competitions.add_registration(&id, &registration).await?;

    Ok(Json(json!({ "success": true, "message": "Successfully registered" })))
}

/// POST /api/v1/competitions/<id>/interest  (auth) — toggle "I'm interested"
#[post("/<id>/interest")]
async fn toggle_interest(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let comp = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;

    let already = comp.interested_user_ids.iter().any(|u| u == &auth.0.sub);
    if already {
        state.competitions.remove_interested(&id, &auth.0.sub).await?;
    } else {
        state.competitions.add_interested(&id, &auth.0.sub).await?;
    }
    Ok(Json(json!({ "interested": !already })))
}

#[derive(Debug, serde::Deserialize)]
struct SetWinnersRequest {
    winner_user_ids: Vec<String>,
}

/// POST /api/v1/competitions/<id>/winners  (admin only)
#[post("/<id>/winners", data = "<req>")]
async fn set_winners(
    id: String,
    req: Json<SetWinnersRequest>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    state
        .competitions
        .set_winners(&id, req.0.winner_user_ids.clone())
        .await?;
    Ok(Json(json!({ "success": true, "count": req.0.winner_user_ids.len() })))
}

pub fn routes() -> Vec<Route> {
    routes![
        list_competitions,
        featured_competitions,
        get_competition,
        create_competition,
        register_competition,
        toggle_interest,
        set_winners
    ]
}
