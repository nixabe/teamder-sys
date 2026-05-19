use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::competition::{
        Competition, CompetitionResponse, CreateCompetitionRequest,
        PublishStatus, RegisterCompetitionRequest, Registration,
        RejectCompetitionRequest,
    },
};
use chrono::Utc;

use crate::{error::ApiResult, guards::{AdminUser, AuthUser, PublisherUser}, state::AppState};

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

/// GET /api/v1/competitions/mine  (publisher or admin)
#[get("/mine")]
async fn list_mine(publisher: PublisherUser, state: &State<AppState>) -> ApiResult<Value> {
    let comps: Vec<CompetitionResponse> = state
        .competitions
        .list_by_publisher(&publisher.0.sub)
        .await?
        .into_iter()
        .map(CompetitionResponse::from)
        .collect();

    Ok(Json(json!({ "data": comps })))
}

/// GET /api/v1/competitions/pending  (admin only)
#[get("/pending")]
async fn list_pending(_admin: AdminUser, state: &State<AppState>) -> ApiResult<Value> {
    let comps: Vec<CompetitionResponse> = state
        .competitions
        .list_pending()
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

/// POST /api/v1/competitions  (admin → Published immediately; publisher → Draft)
#[post("/", data = "<req>")]
async fn create_competition(
    req: Json<CreateCompetitionRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<CompetitionResponse> {
    if !auth.0.is_admin && !auth.0.is_publisher {
        return Err(TeamderError::Forbidden.into());
    }

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
    if req.banner_image.is_some() { comp.banner_image = req.banner_image.clone(); }

    if auth.0.is_admin {
        comp.publish_status = PublishStatus::Published;
    } else {
        comp.publish_status = PublishStatus::Draft;
        comp.publisher_id = Some(auth.0.sub.clone());
    }

    state.competitions.create(&comp).await?;
    Ok(Json(comp.into()))
}

/// POST /api/v1/competitions/<id>/submit-review  (auth — must own the competition)
#[post("/<id>/submit-review")]
async fn submit_review(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let comp = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;

    // Ownership check: must be the publisher or an admin
    let is_owner = comp.publisher_id.as_deref() == Some(&auth.0.sub);
    if !auth.0.is_admin && !is_owner {
        return Err(TeamderError::Forbidden.into());
    }

    if comp.publish_status != PublishStatus::Draft {
        return Err(TeamderError::Validation(
            "Only draft competitions can be submitted for review".into()
        ).into());
    }

    state.competitions.set_publish_status(&id, &PublishStatus::PendingReview, None).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/competitions/<id>/approve  (admin only)
#[post("/<id>/approve")]
async fn approve_competition(
    id: String,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let comp = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;

    if comp.publish_status != PublishStatus::PendingReview {
        return Err(TeamderError::Validation(
            "Only pending-review competitions can be approved".into()
        ).into());
    }

    state.competitions.set_publish_status(&id, &PublishStatus::Published, None).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/competitions/<id>/reject  (admin only)
#[post("/<id>/reject", data = "<req>")]
async fn reject_competition(
    id: String,
    req: Json<RejectCompetitionRequest>,
    _admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let comp = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;

    if comp.publish_status != PublishStatus::PendingReview {
        return Err(TeamderError::Validation(
            "Only pending-review competitions can be rejected".into()
        ).into());
    }

    let note = req.0.note.as_deref();
    state.competitions.set_publish_status(&id, &PublishStatus::Rejected, note).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/competitions/<id>/register  (auth)
#[post("/<id>/register", data = "<req>")]
async fn register_competition(
    id: String,
    req: Json<RegisterCompetitionRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
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

/// PATCH /api/v1/competitions/<id>  (admin: anything; publisher: own Draft/Rejected only)
#[patch("/<id>", data = "<req>")]
async fn update_competition(
    id: String,
    req: Json<Value>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    use mongodb::bson::{Document, Bson};

    if !auth.0.is_admin && !auth.0.is_publisher {
        return Err(TeamderError::Forbidden.into());
    }

    let comp = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;

    if !auth.0.is_admin {
        let is_owner = comp.publisher_id.as_deref() == Some(&auth.0.sub);
        if !is_owner {
            return Err(TeamderError::Forbidden.into());
        }
        if comp.publish_status != PublishStatus::Draft && comp.publish_status != PublishStatus::Rejected {
            return Err(TeamderError::Validation(
                "Only draft or rejected competitions can be edited".into()
            ).into());
        }
    }

    let allowed = ["name","organizer","description","prize","team_size_min","team_size_max",
                   "deadline","duration","tags","is_featured","status","icon","icon_bg","banner_image"];
    let mut patch = Document::new();
    if let Some(obj) = req.0.as_object() {
        for key in allowed {
            if let Some(val) = obj.get(key) {
                if let Ok(bson_val) = mongodb::bson::to_bson(val) {
                    patch.insert(key, bson_val);
                }
            }
        }
    }
    if patch.is_empty() {
        return Ok(Json(json!({ "success": true })));
    }
    patch.insert("updated_at", Bson::String(Utc::now().to_rfc3339()));
    state.competitions.update(&id, patch).await?;
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/v1/competitions/<id>  (admin: anything; publisher: own Draft only)
#[delete("/<id>")]
async fn delete_competition(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    if !auth.0.is_admin && !auth.0.is_publisher {
        return Err(TeamderError::Forbidden.into());
    }

    let comp = state
        .competitions
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Competition {} not found", id)))?;

    if !auth.0.is_admin {
        let is_owner = comp.publisher_id.as_deref() == Some(&auth.0.sub);
        if !is_owner {
            return Err(TeamderError::Forbidden.into());
        }
        if comp.publish_status != PublishStatus::Draft {
            return Err(TeamderError::Validation(
                "Only draft competitions can be deleted by publishers".into()
            ).into());
        }
    }

    state.competitions.delete(&id).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![
        list_competitions,
        featured_competitions,
        list_mine,
        list_pending,
        get_competition,
        create_competition,
        submit_review,
        approve_competition,
        reject_competition,
        update_competition,
        delete_competition,
        register_competition,
        toggle_interest,
        set_winners
    ]
}
