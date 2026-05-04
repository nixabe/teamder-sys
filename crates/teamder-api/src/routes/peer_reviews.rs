use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::{
        peer_review::{CreatePeerReviewRequest, PeerReview, PeerReviewResponse},
        user::Review,
    },
    skills::filter_valid_skills,
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// POST /api/v1/reviews
///
/// Submit a peer review of another user. Both reviewer and reviewee must have
/// been on the same project (when project_id is supplied). Each (reviewer,
/// reviewee, project) triplet may only be used once.
#[post("/", data = "<req>")]
async fn create_review(
    req: Json<CreatePeerReviewRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<PeerReviewResponse> {
    let reviewer_id = auth.0.sub.clone();

    if reviewer_id == req.reviewee_id {
        return Err(TeamderError::Validation("Cannot review yourself".into()).into());
    }

    let reviewer = state
        .users
        .find_by_id(&reviewer_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Reviewer not found".into()))?;

    let reviewee = state
        .users
        .find_by_id(&req.reviewee_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Reviewee not found".into()))?;

    // If a project is referenced, ensure both parties were on it.
    if let Some(pid) = &req.project_id {
        let project = state
            .projects
            .find_by_id(pid)
            .await?
            .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;
        let on_project = |uid: &str| {
            project.lead_user_id == uid || project.team.iter().any(|m| m.user_id == uid)
        };
        if !on_project(&reviewer_id) || !on_project(&req.reviewee_id) {
            return Err(TeamderError::Forbidden.into());
        }
    }

    // Prevent duplicate reviews for the same pair + project.
    let exists = state
        .peer_reviews
        .exists_pair(&reviewer_id, &req.reviewee_id, req.project_id.as_deref())
        .await?;
    if exists {
        return Err(TeamderError::Conflict("You already reviewed this collaborator for this project".into()).into());
    }

    let mut scores = req.0.scores;
    scores.clamp();

    let body = req.0.body.trim().to_string();
    if body.len() < 5 {
        return Err(TeamderError::Validation("Review body must be at least 5 characters".into()).into());
    }

    let endorsed = filter_valid_skills(&req.0.endorsed_skills);

    let review = PeerReview::new(
        reviewer_id,
        reviewer.name.clone(),
        req.0.reviewee_id.clone(),
        req.0.project_id.clone(),
        req.0.project_name.clone(),
        scores,
        body,
        endorsed,
    );
    state.peer_reviews.create(&review).await?;

    // Refresh aggregate cached on reviewee.
    let (avg, count) = state.peer_reviews.average_for_user(&reviewee.id).await?;
    state.users.set_rating(&reviewee.id, avg, count).await?;

    // Push embedded review for fast display.
    let stars = scores.average().round() as u8;
    let embedded = Review {
        reviewer_id: review.reviewer_id.clone(),
        reviewer_name: review.reviewer_name.clone(),
        project_name: review.project_name.clone(),
        stars: stars.clamp(1, 5),
        body: review.body.clone(),
        created_at: review.created_at,
    };
    state.users.push_review(&reviewee.id, &embedded).await?;

    Ok(Json(review.into()))
}

/// GET /api/v1/reviews/user/<id> — list reviews left FOR a user.
#[get("/user/<id>")]
async fn list_for_user(
    id: String,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let reviews = state.peer_reviews.list_for_user(&id).await?;
    let data: Vec<PeerReviewResponse> = reviews.into_iter().map(Into::into).collect();
    let avg: f32 = if data.is_empty() {
        0.0
    } else {
        data.iter().map(|r| r.average).sum::<f32>() / data.len() as f32
    };
    Ok(Json(json!({ "data": data, "average": avg, "count": data.len() })))
}

/// GET /api/v1/reviews/mine — reviews YOU have written.
#[get("/mine")]
async fn list_mine(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let reviews = state.peer_reviews.list_by_reviewer(&auth.0.sub).await?;
    let data: Vec<PeerReviewResponse> = reviews.into_iter().map(Into::into).collect();
    Ok(Json(json!({ "data": data })))
}

pub fn routes() -> Vec<Route> {
    routes![create_review, list_for_user, list_mine]
}
