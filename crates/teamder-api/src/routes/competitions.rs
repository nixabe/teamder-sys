use chrono::Utc;
use mongodb::bson;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::{AdminUser, AuthUser, OptionalAuth, PublisherUser};
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::competition::{
    Competition, CompetitionResponse, CreateCompetitionRequest, Registration,
    UpdateCompetitionRequest,
};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PaginatedCompetitions {
    pub competitions: Vec<CompetitionResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: i64,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    #[serde(default)]
    pub team_name: Option<String>,
    #[serde(default)]
    pub motivation: Option<String>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub contact_email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InterestResponse {
    pub interested: bool,
}

#[derive(Debug, Deserialize)]
pub struct WinnersBody {
    pub winner_user_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectBody {
    #[serde(default)]
    pub note: Option<String>,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/competitions?<page>&<limit>&<status>&<q>")]
pub async fn list_competitions(
    state: &State<AppState>,
    page: Option<u64>,
    limit: Option<i64>,
    status: Option<String>,
    q: Option<String>,
) -> Result<Json<PaginatedCompetitions>, ApiError> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);
    let skip = (page.saturating_sub(1)) * (limit as u64);

    // For public listing, always filter by published
    let mut filter = bson::doc! { "publish_status": "published" };
    if let Some(s) = &status {
        filter.insert("status", s.as_str());
    }

    let (comps, _total) = state
        .db
        .competition_repo()
        .list(status.as_deref(), q.as_deref(), skip, limit)
        .await?;

    // Filter published only for public listing
    let comps: Vec<CompetitionResponse> = comps
        .into_iter()
        .filter(|c| c.publish_status == "published")
        .map(|c| CompetitionResponse::from_competition(c, None, false))
        .collect();
    let total = comps.len() as u64;

    Ok(Json(PaginatedCompetitions {
        competitions: comps,
        total,
        page,
        limit,
    }))
}

#[rocket::get("/competitions/featured")]
pub async fn featured_competitions(
    state: &State<AppState>,
) -> Result<Json<Vec<CompetitionResponse>>, ApiError> {
    let comps = state.db.competition_repo().featured().await?;
    let resp: Vec<CompetitionResponse> = comps
        .into_iter()
        .map(|c| CompetitionResponse::from_competition(c, None, false))
        .collect();
    Ok(Json(resp))
}

#[rocket::get("/competitions/mine")]
pub async fn my_competitions(
    state: &State<AppState>,
    auth: PublisherUser,
) -> Result<Json<Vec<CompetitionResponse>>, ApiError> {
    let comps = state
        .db
        .competition_repo()
        .find_by_publisher(&auth.user_id)
        .await?;
    let resp: Vec<CompetitionResponse> = comps
        .into_iter()
        .map(|c| CompetitionResponse::from_competition(c, Some(&auth.user_id), true))
        .collect();
    Ok(Json(resp))
}

#[rocket::get("/competitions/pending")]
pub async fn pending_competitions(
    state: &State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<CompetitionResponse>>, ApiError> {
    let comps = state.db.competition_repo().find_pending().await?;
    let resp: Vec<CompetitionResponse> = comps
        .into_iter()
        .map(|c| CompetitionResponse::from_competition(c, None, true))
        .collect();
    Ok(Json(resp))
}

#[rocket::get("/competitions/<id>")]
pub async fn get_competition(
    state: &State<AppState>,
    id: &str,
    viewer: OptionalAuth,
) -> Result<Json<CompetitionResponse>, ApiError> {
    let comp = state
        .db
        .competition_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

    let viewer_id = viewer.0.as_deref();
    let include_regs = viewer_id
        .map(|vid| {
            comp.publisher_id.as_deref() == Some(vid)
        })
        .unwrap_or(false);

    Ok(Json(CompetitionResponse::from_competition(
        comp,
        viewer_id,
        include_regs,
    )))
}

#[rocket::post("/competitions", data = "<body>")]
pub async fn create_competition(
    state: &State<AppState>,
    auth: PublisherUser,
    body: Json<CreateCompetitionRequest>,
) -> Result<Json<CompetitionResponse>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    let comp = Competition {
        id: id.clone(),
        name: req.name,
        organizer: req.organizer,
        icon: req.icon.unwrap_or_else(|| "Cp".to_string()),
        icon_bg: req.icon_bg.unwrap_or_default(),
        status: req.status.unwrap_or_else(|| "open".to_string()),
        prize: req.prize.unwrap_or_default(),
        team_size_min: req.team_size_min.unwrap_or(2),
        team_size_max: req.team_size_max.unwrap_or(5),
        deadline: req.deadline,
        duration: req.duration.unwrap_or_default(),
        tags: req.tags.unwrap_or_default(),
        description: req.description.unwrap_or_default(),
        is_featured: req.is_featured.unwrap_or(false),
        banner_image: req.banner_image,
        publish_status: req.publish_status.unwrap_or_else(|| "draft".to_string()),
        publisher_id: Some(auth.user_id.clone()),
        rejected_note: None,
        registrations: vec![],
        interested_user_ids: vec![],
        winners: vec![],
        created_at: now,
        updated_at: now,
    };

    state.db.competition_repo().create(&comp).await?;

    Ok(Json(CompetitionResponse::from_competition(
        comp,
        Some(&auth.user_id),
        true,
    )))
}

#[rocket::patch("/competitions/<id>", data = "<body>")]
pub async fn update_competition(
    state: &State<AppState>,
    auth: PublisherUser,
    id: &str,
    body: Json<UpdateCompetitionRequest>,
) -> Result<Json<CompetitionResponse>, ApiError> {
    let comp = state
        .db
        .competition_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

    // Must be the owner or admin
    let caller = state.db.user_repo().find_by_id(&auth.user_id).await?;
    let is_owner = comp.publisher_id.as_deref() == Some(&auth.user_id);
    let is_admin = caller.map(|u| u.is_admin).unwrap_or(false);
    if !is_owner && !is_admin {
        return Err(TeamderError::Forbidden("Not authorized".into()).into());
    }

    let req = body.into_inner();
    let mut update = bson::doc! {};

    if let Some(v) = &req.name { update.insert("name", v.as_str()); }
    if let Some(v) = &req.organizer { update.insert("organizer", v.as_str()); }
    if let Some(v) = &req.icon { update.insert("icon", v.as_str()); }
    if let Some(v) = &req.icon_bg { update.insert("icon_bg", v.as_str()); }
    if let Some(v) = &req.status { update.insert("status", v.as_str()); }
    if let Some(v) = &req.prize { update.insert("prize", v.as_str()); }
    if let Some(v) = req.team_size_min { update.insert("team_size_min", v as i32); }
    if let Some(v) = req.team_size_max { update.insert("team_size_max", v as i32); }
    if let Some(v) = &req.deadline { update.insert("deadline", v.as_str()); }
    if let Some(v) = &req.duration { update.insert("duration", v.as_str()); }
    if let Some(v) = &req.tags {
        update.insert("tags", bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?);
    }
    if let Some(v) = &req.description { update.insert("description", v.as_str()); }
    if let Some(v) = req.is_featured { update.insert("is_featured", v); }
    if let Some(v) = &req.banner_image { update.insert("banner_image", v.as_str()); }
    if let Some(v) = &req.publish_status { update.insert("publish_status", v.as_str()); }
    if let Some(v) = &req.rejected_note { update.insert("rejected_note", v.as_str()); }

    update.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));

    state.db.competition_repo().update(id, update).await?;

    let updated = state
        .db
        .competition_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

    Ok(Json(CompetitionResponse::from_competition(
        updated,
        Some(&auth.user_id),
        true,
    )))
}

#[rocket::post("/competitions/<id>/register", data = "<body>")]
pub async fn register_competition(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<RegisterBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let comp = state
        .db
        .competition_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

    // Check not already registered
    if comp.registrations.iter().any(|r| r.user_id == auth.user_id) {
        return Err(TeamderError::Conflict("Already registered".into()).into());
    }

    let req = body.into_inner();
    let registration = Registration {
        user_id: auth.user_id,
        team_name: req.team_name,
        registered_at: Utc::now(),
        motivation: req.motivation,
        skills: req.skills,
        contact_email: req.contact_email,
    };

    state
        .db
        .competition_repo()
        .register_user(id, &registration)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/competitions/<id>/interest")]
pub async fn toggle_interest(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<InterestResponse>, ApiError> {
    let interested = state
        .db
        .competition_repo()
        .toggle_interest(id, &auth.user_id)
        .await?;

    Ok(Json(InterestResponse { interested }))
}

#[rocket::get("/competitions/<id>/registrations")]
pub async fn get_registrations(
    state: &State<AppState>,
    auth: PublisherUser,
    id: &str,
) -> Result<Json<Vec<Registration>>, ApiError> {
    let comp = state
        .db
        .competition_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

    // Must be owner or admin
    let caller = state.db.user_repo().find_by_id(&auth.user_id).await?;
    let is_owner = comp.publisher_id.as_deref() == Some(&auth.user_id);
    let is_admin = caller.map(|u| u.is_admin).unwrap_or(false);
    if !is_owner && !is_admin {
        return Err(TeamderError::Forbidden("Not authorized".into()).into());
    }

    Ok(Json(comp.registrations))
}

#[rocket::post("/competitions/<id>/submit-review")]
pub async fn submit_review(
    state: &State<AppState>,
    auth: PublisherUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let comp = state
        .db
        .competition_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Competition not found".into()))?;

    if comp.publisher_id.as_deref() != Some(&auth.user_id) {
        return Err(TeamderError::Forbidden("Not the publisher".into()).into());
    }

    if comp.publish_status != "draft" {
        return Err(TeamderError::Validation("Only drafts can be submitted for review".into()).into());
    }

    state
        .db
        .competition_repo()
        .set_publish_status(id, "pending_review", None)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/competitions/<id>/approve")]
pub async fn approve_competition(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    state
        .db
        .competition_repo()
        .set_publish_status(id, "published", None)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/competitions/<id>/reject", data = "<body>")]
pub async fn reject_competition(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
    body: Json<RejectBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let req = body.into_inner();
    state
        .db
        .competition_repo()
        .set_publish_status(id, "rejected", req.note.as_deref())
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/competitions/<id>/winners", data = "<body>")]
pub async fn set_winners(
    state: &State<AppState>,
    _admin: AdminUser,
    id: &str,
    body: Json<WinnersBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    state
        .db
        .competition_repo()
        .set_winners(id, &body.winner_user_ids)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}
