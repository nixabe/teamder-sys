use chrono::{DateTime, Duration, Utc};
use rocket::{serde::json::Json, Route, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use teamder_core::{
    error::TeamderError,
    models::{
        notification::{Notification, NotificationKind},
        peer_review::{CreatePeerReviewRequest, PeerReview, PeerReviewResponse},
        project::{Project, ProjectStatus},
        study_group::{StudyGroup, StudyGroupStatus},
        user::Review,
    },
    skills::filter_valid_skills,
};

use crate::{
    error::ApiResult,
    guards::AuthUser,
    llm::{ReviewAssistContext, ReviewQa},
    state::AppState,
};

#[derive(Debug, Deserialize)]
struct ReviewAssistRequest {
    reviewee_id: String,
    project_name: String,
    scores: teamder_core::models::peer_review::ReviewScores,
    initial_body: String,
    #[serde(default)]
    answers: Vec<ReviewQa>,
    #[serde(default)]
    clarification_note: Option<String>,
}

#[derive(Debug, Serialize)]
struct ReviewAssistQuestionsResponse {
    questions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReviewAssistSummaryResponse {
    summary: String,
}

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

    if req.project_id.is_none() && req.study_group_id.is_none() {
        return Err(TeamderError::Validation(
            "Either project_id or study_group_id is required".into(),
        )
        .into());
    }
    let min_collab_days = review_min_collab_days();

    // If a project is referenced, ensure both parties were on it and it's completed.
    if let Some(pid) = &req.project_id {
        let project = state
            .projects
            .find_by_id(pid)
            .await?
            .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;
        if project.status != ProjectStatus::Completed {
            return Err(TeamderError::Validation(
                "Reviews can only be submitted after the project is completed".into(),
            )
            .into());
        }
        let on_project = |uid: &str| {
            project.lead_user_id == uid || project.team.iter().any(|m| m.user_id == uid)
        };
        if !on_project(&reviewer_id) || !on_project(&req.reviewee_id) {
            return Err(TeamderError::Forbidden.into());
        }
        ensure_review_collaboration_age(
            min_collab_days,
            project_participant_since(&project, &reviewer_id),
            project_participant_since(&project, &req.reviewee_id),
        )?;
    }

    // If a study group is referenced, ensure both parties were in it and it's completed.
    if let Some(gid) = &req.study_group_id {
        let group = state
            .study_groups
            .find_by_id(gid)
            .await?
            .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;
        if group.status != StudyGroupStatus::Completed {
            return Err(TeamderError::Validation(
                "Reviews can only be submitted after the study group is completed".into(),
            )
            .into());
        }
        let in_group =
            |uid: &str| group.created_by == uid || group.members.iter().any(|m| m.user_id == uid);
        if !in_group(&reviewer_id) || !in_group(&req.reviewee_id) {
            return Err(TeamderError::Forbidden.into());
        }
        ensure_review_collaboration_age(
            min_collab_days,
            study_group_participant_since(&group, &reviewer_id),
            study_group_participant_since(&group, &req.reviewee_id),
        )?;
    }

    // Prevent duplicate reviews for the same pair + project.
    let exists = state
        .peer_reviews
        .exists_pair(
            &reviewer_id,
            &req.reviewee_id,
            req.project_id.as_deref(),
            req.study_group_id.as_deref(),
        )
        .await?;
    if exists {
        return Err(TeamderError::Conflict(
            "You already reviewed this collaborator for this project".into(),
        )
        .into());
    }

    let mut scores = req.0.scores;
    scores.clamp();

    let body = req.0.body.trim().to_string();
    if body.len() < 5 {
        return Err(
            TeamderError::Validation("Review body must be at least 5 characters".into()).into(),
        );
    }

    let endorsed = filter_valid_skills(&req.0.endorsed_skills);

    let review = PeerReview::new(
        reviewer_id,
        reviewer.name.clone(),
        req.0.reviewee_id.clone(),
        req.0.project_id.clone(),
        req.0.study_group_id.clone(),
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

    // Notify the reviewee.
    let n = Notification::new(
        reviewee.id.clone(),
        NotificationKind::Review,
        "New peer review",
        format!(
            "{} left you a review on {}",
            reviewer.name, review.project_name
        ),
        Some(format!("/profile/{}", reviewee.id)),
    );
    if let Err(e) = state.notifications.create(&n).await {
        tracing::warn!("failed to create review notification: {e}");
    }

    Ok(Json(review.into()))
}

/// POST /api/v1/reviews/assist/questions
///
/// Ask the configured LLM for 2-3 clarification questions based on the
/// commenter's draft and any previous answers.
#[post("/assist/questions", data = "<req>")]
async fn assist_questions(
    req: Json<ReviewAssistRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<ReviewAssistQuestionsResponse> {
    let (reviewer_name, reviewee_name) =
        review_assist_names(&auth.0.sub, &req.reviewee_id, state).await?;
    let initial_body = req.initial_body.trim();
    if initial_body.len() < 5 {
        return Err(TeamderError::Validation(
            "Initial comment must be at least 5 characters".into(),
        )
        .into());
    }

    let questions = state
        .review_llm
        .clarification_questions(ReviewAssistContext {
            reviewer_name: &reviewer_name,
            reviewee_name: &reviewee_name,
            project_name: req.project_name.trim(),
            scores: req.scores,
            initial_body,
            answers: &req.answers,
            clarification_note: req.clarification_note.as_deref(),
        })
        .await?;

    Ok(Json(ReviewAssistQuestionsResponse { questions }))
}

/// POST /api/v1/reviews/assist/summary
///
/// Summarize the initial comment and clarification answers into the review body
/// that the commenter previews before submitting.
#[post("/assist/summary", data = "<req>")]
async fn assist_summary(
    req: Json<ReviewAssistRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<ReviewAssistSummaryResponse> {
    let (reviewer_name, reviewee_name) =
        review_assist_names(&auth.0.sub, &req.reviewee_id, state).await?;
    let initial_body = req.initial_body.trim();
    if initial_body.len() < 5 {
        return Err(TeamderError::Validation(
            "Initial comment must be at least 5 characters".into(),
        )
        .into());
    }

    let summary = state
        .review_llm
        .summarize_review(ReviewAssistContext {
            reviewer_name: &reviewer_name,
            reviewee_name: &reviewee_name,
            project_name: req.project_name.trim(),
            scores: req.scores,
            initial_body,
            answers: &req.answers,
            clarification_note: req.clarification_note.as_deref(),
        })
        .await?;

    Ok(Json(ReviewAssistSummaryResponse { summary }))
}

async fn review_assist_names(
    reviewer_id: &str,
    reviewee_id: &str,
    state: &State<AppState>,
) -> Result<(String, String), TeamderError> {
    if reviewer_id == reviewee_id {
        return Err(TeamderError::Validation("Cannot review yourself".into()));
    }

    let reviewer = state
        .users
        .find_by_id(reviewer_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Reviewer not found".into()))?;
    let reviewee = state
        .users
        .find_by_id(reviewee_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Reviewee not found".into()))?;

    Ok((reviewer.name, reviewee.name))
}

fn review_min_collab_days() -> i64 {
    std::env::var("REVIEW_MIN_COLLAB_DAYS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0)
}

fn project_participant_since(project: &Project, user_id: &str) -> Option<DateTime<Utc>> {
    if project.lead_user_id == user_id {
        return Some(project.created_at);
    }
    project
        .team
        .iter()
        .find(|member| member.user_id == user_id)
        .map(|member| member.joined_at)
}

fn study_group_participant_since(group: &StudyGroup, user_id: &str) -> Option<DateTime<Utc>> {
    if group.created_by == user_id {
        return Some(group.created_at);
    }
    group
        .members
        .iter()
        .find(|member| member.user_id == user_id)
        .map(|member| member.joined_at)
}

fn ensure_review_collaboration_age(
    min_days: i64,
    reviewer_since: Option<DateTime<Utc>>,
    reviewee_since: Option<DateTime<Utc>>,
) -> Result<(), TeamderError> {
    if min_days <= 0 {
        return Ok(());
    }

    let min_duration = Duration::days(min_days);
    let now = Utc::now();
    let reviewer_since = reviewer_since.ok_or(TeamderError::Forbidden)?;
    let reviewee_since = reviewee_since.ok_or(TeamderError::Forbidden)?;

    if now.signed_duration_since(reviewer_since) < min_duration
        || now.signed_duration_since(reviewee_since) < min_duration
    {
        return Err(TeamderError::Validation(format!(
            "Reviews require at least {min_days} days of shared collaboration"
        )));
    }

    Ok(())
}

/// GET /api/v1/reviews/user/<id> — list reviews left FOR a user.
#[get("/user/<id>")]
async fn list_for_user(id: String, state: &State<AppState>) -> ApiResult<Value> {
    let reviews = state.peer_reviews.list_for_user(&id).await?;
    let data: Vec<PeerReviewResponse> = reviews.into_iter().map(Into::into).collect();
    let avg: f32 = if data.is_empty() {
        0.0
    } else {
        data.iter().map(|r| r.average).sum::<f32>() / data.len() as f32
    };
    Ok(Json(
        json!({ "data": data, "average": avg, "count": data.len() }),
    ))
}

/// GET /api/v1/reviews/mine — reviews YOU have written.
#[get("/mine")]
async fn list_mine(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let reviews = state.peer_reviews.list_by_reviewer(&auth.0.sub).await?;
    let data: Vec<PeerReviewResponse> = reviews.into_iter().map(Into::into).collect();
    Ok(Json(json!({ "data": data })))
}

pub fn routes() -> Vec<Route> {
    routes![
        create_review,
        assist_questions,
        assist_summary,
        list_for_user,
        list_mine
    ]
}
