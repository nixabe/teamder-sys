use chrono::Utc;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::notification::Notification;
use teamder_core::models::peer_review::{PeerReview, ReviewScores};
use teamder_core::models::user::CachedReview;

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateReviewBody {
    pub reviewee_id: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub study_group_id: Option<String>,
    #[serde(default)]
    pub project_name: Option<String>,
    pub scores: ReviewScores,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub endorsed_skills: Option<Vec<String>>,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/reviews/for/<user_id>")]
pub async fn reviews_for_user(
    state: &State<AppState>,
    user_id: &str,
) -> Result<Json<Vec<PeerReview>>, ApiError> {
    let reviews = state
        .db
        .peer_review_repo()
        .find_for_user(user_id)
        .await?;
    Ok(Json(reviews))
}

#[rocket::post("/reviews", data = "<body>")]
pub async fn create_review(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<CreateReviewBody>,
) -> Result<Json<PeerReview>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();

    let reviewer = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Reviewer not found".into()))?;

    let review = PeerReview {
        id: Uuid::new_v4().to_string(),
        reviewer_id: auth.user_id.clone(),
        reviewer_name: reviewer.name.clone(),
        reviewee_id: req.reviewee_id.clone(),
        project_id: req.project_id,
        study_group_id: req.study_group_id,
        project_name: req.project_name.clone().unwrap_or_default(),
        scores: req.scores,
        body: req.body.clone().unwrap_or_default(),
        endorsed_skills: req.endorsed_skills.unwrap_or_default(),
        created_at: now,
    };

    state.db.peer_review_repo().create(&review).await?;

    // Recalculate reviewee average rating
    let all_reviews = state
        .db
        .peer_review_repo()
        .find_for_user(&req.reviewee_id)
        .await?;

    let total: f32 = all_reviews.iter().map(|r| r.scores.average()).sum();
    let avg = if all_reviews.is_empty() {
        0.0
    } else {
        total / all_reviews.len() as f32
    };

    // Push cached review + update rating
    let cached = CachedReview {
        reviewer_id: auth.user_id.clone(),
        reviewer_name: reviewer.name,
        project_name: req.project_name.unwrap_or_default(),
        stars: (review.scores.average().round()) as u8,
        body: req.body.unwrap_or_default(),
        created_at: now,
    };

    state
        .db
        .user_repo()
        .update_rating(&req.reviewee_id, avg, &cached)
        .await?;

    // Create notification
    let notif = Notification {
        id: Uuid::new_v4().to_string(),
        user_id: req.reviewee_id,
        kind: "review".to_string(),
        title: "You received a new peer review".to_string(),
        body: String::new(),
        link: None,
        read: false,
        created_at: now,
    };
    let _ = state.db.notification_repo().create(&notif).await;

    Ok(Json(review))
}
