use std::collections::HashMap;

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
    #[serde(default)]
    context_type: Option<String>,
    #[serde(default)]
    context_id: Option<String>,
    project_name: String,
    #[serde(default)]
    language: Option<String>,
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

const REVIEW_ASSIST_MIN_INPUT_CHARS: usize = 25;

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
    validate_review_assist_inputs(initial_body, &req.answers)?;
    let context_details = review_context_details(
        req.context_type.as_deref(),
        req.context_id.as_deref(),
        &auth.0.sub,
        &req.reviewee_id,
        state,
    )
    .await?;

    let questions = state
        .review_llm
        .clarification_questions(ReviewAssistContext {
            reviewer_name: &reviewer_name,
            reviewee_name: &reviewee_name,
            project_name: req.project_name.trim(),
            context_details: context_details.as_deref(),
            language: review_assist_language(req.language.as_deref()),
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
    validate_review_assist_inputs(initial_body, &req.answers)?;
    let context_details = review_context_details(
        req.context_type.as_deref(),
        req.context_id.as_deref(),
        &auth.0.sub,
        &req.reviewee_id,
        state,
    )
    .await?;

    let summary = state
        .review_llm
        .summarize_review(ReviewAssistContext {
            reviewer_name: &reviewer_name,
            reviewee_name: &reviewee_name,
            project_name: req.project_name.trim(),
            context_details: context_details.as_deref(),
            language: review_assist_language(req.language.as_deref()),
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

fn review_assist_language(language: Option<&str>) -> &str {
    match language.map(str::trim).filter(|s| !s.is_empty()) {
        Some("zh-TW") | Some("Traditional Chinese") | Some("繁體中文") => "Traditional Chinese",
        Some("zh-CN") | Some("Simplified Chinese") | Some("简体中文") => "Simplified Chinese",
        Some("en") | Some("English") => "English",
        Some(other) => other,
        None => "the same language as the commenter's input",
    }
}

fn validate_review_assist_inputs(
    initial_body: &str,
    answers: &[ReviewQa],
) -> Result<(), TeamderError> {
    if initial_body.trim().chars().count() < REVIEW_ASSIST_MIN_INPUT_CHARS {
        return Err(TeamderError::Validation(format!(
            "Initial comment must be at least {REVIEW_ASSIST_MIN_INPUT_CHARS} characters"
        )));
    }

    if answers.iter().any(|qa| {
        !qa.question.trim().is_empty()
            && qa.answer.trim().chars().count() < REVIEW_ASSIST_MIN_INPUT_CHARS
    }) {
        return Err(TeamderError::Validation(format!(
            "Each clarification answer must be at least {REVIEW_ASSIST_MIN_INPUT_CHARS} characters"
        )));
    }

    Ok(())
}

async fn review_context_details(
    context_type: Option<&str>,
    context_id: Option<&str>,
    reviewer_id: &str,
    reviewee_id: &str,
    state: &State<AppState>,
) -> Result<Option<String>, TeamderError> {
    let Some(context_id) = context_id.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };

    match context_type.map(str::trim) {
        Some("project") => {
            let project = state
                .projects
                .find_by_id(context_id)
                .await?
                .ok_or_else(|| TeamderError::NotFound("Project not found".into()))?;
            let on_project = |uid: &str| {
                project.lead_user_id == uid || project.team.iter().any(|m| m.user_id == uid)
            };
            if !on_project(reviewer_id) || !on_project(reviewee_id) {
                return Err(TeamderError::Forbidden);
            }
            project_context_details(&project, state).await.map(Some)
        }
        Some("study_group") => {
            let group = state
                .study_groups
                .find_by_id(context_id)
                .await?
                .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;
            let in_group = |uid: &str| {
                group.created_by == uid || group.members.iter().any(|m| m.user_id == uid)
            };
            if !in_group(reviewer_id) || !in_group(reviewee_id) {
                return Err(TeamderError::Forbidden);
            }
            study_group_context_details(&group, state).await.map(Some)
        }
        _ => Ok(None),
    }
}

async fn project_context_details(
    project: &Project,
    state: &State<AppState>,
) -> Result<String, TeamderError> {
    let mut ids: Vec<&str> = vec![project.lead_user_id.as_str()];
    ids.extend(project.team.iter().map(|m| m.user_id.as_str()));
    let users = state.users.find_many_by_ids(&ids).await?;
    let names: HashMap<&str, &str> = users
        .iter()
        .map(|u| (u.id.as_str(), u.name.as_str()))
        .collect();

    let roles = project
        .roles
        .iter()
        .map(|r| {
            format!(
                "{} (needed {}, filled {})",
                r.name, r.count_needed, r.filled
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let team = project
        .team
        .iter()
        .map(|m| {
            let name = names
                .get(m.user_id.as_str())
                .copied()
                .unwrap_or(m.initials.as_str());
            match &m.role {
                Some(role) if !role.trim().is_empty() => format!("{name} - {role}"),
                _ => name.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    let lead_name = names
        .get(project.lead_user_id.as_str())
        .copied()
        .unwrap_or("Unknown lead");

    Ok(format!(
        "Type: project\nTitle: {}\nDescription: {}\nGoals: {}\nSkills: {}\nOpen roles: {}\nLead: {}\nTeam members: {}",
        project.name,
        project.description,
        project.goals.as_deref().unwrap_or("Not specified"),
        if project.skills.is_empty() { "Not specified".to_string() } else { project.skills.join(", ") },
        if roles.is_empty() { "Not specified".to_string() } else { roles },
        lead_name,
        if team.is_empty() { "No listed members".to_string() } else { team },
    ))
}

async fn study_group_context_details(
    group: &StudyGroup,
    state: &State<AppState>,
) -> Result<String, TeamderError> {
    let mut ids: Vec<&str> = vec![group.created_by.as_str()];
    ids.extend(group.members.iter().map(|m| m.user_id.as_str()));
    let users = state.users.find_many_by_ids(&ids).await?;
    let names: HashMap<&str, &str> = users
        .iter()
        .map(|u| (u.id.as_str(), u.name.as_str()))
        .collect();

    let creator_name = names
        .get(group.created_by.as_str())
        .copied()
        .unwrap_or("Unknown creator");
    let members = group
        .members
        .iter()
        .map(|m| {
            let name = names
                .get(m.user_id.as_str())
                .copied()
                .unwrap_or(m.initials.as_str());
            format!("{name} (streak {})", m.streak)
        })
        .collect::<Vec<_>>()
        .join("; ");

    Ok(format!(
        "Type: study group\nTitle: {}\nGoal: {}\nDescription: {}\nSubject: {}\nTags: {}\nSchedule: {}\nCreator: {}\nMembers: {}",
        group.name,
        group.goal,
        group.description.as_deref().unwrap_or("Not specified"),
        group.subject,
        if group.tags.is_empty() { "Not specified".to_string() } else { group.tags.join(", ") },
        group.schedule,
        creator_name,
        if members.is_empty() { "No listed members".to_string() } else { members },
    ))
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
