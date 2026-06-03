use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::user::{UpdateUserRequest, User, UserResponse},
    skills::{compute_match_score, filter_valid_skills},
};

use crate::{
    error::ApiResult,
    guards::{AuthUser, OptionalAuth},
    state::AppState,
};

/// Compute match scores for a list of target users from the viewer's perspective.
/// If viewer is None, all scores are returned as 0.
async fn fill_match_scores(
    state: &AppState,
    viewer_id: Option<&str>,
    targets: Vec<User>,
) -> Result<Vec<UserResponse>, TeamderError> {
    let viewer = if let Some(vid) = viewer_id {
        state.users.find_by_id(vid).await?
    } else {
        None
    };

    let viewer_projects = if let Some(v) = &viewer {
        state.projects.list_by_member(&v.id).await.unwrap_or_default()
    } else {
        vec![]
    };

    let mut out = Vec::with_capacity(targets.len());
    for t in targets {
        let target_projects = state.projects.list_by_member(&t.id).await.unwrap_or_default();
        let score = if let Some(v) = &viewer {
            if v.id == t.id {
                t.match_score // self → leave as-is
            } else {
                compute_match_score(v, &t, &viewer_projects, &target_projects)
            }
        } else {
            0
        };
        let mut resp: UserResponse = t.into();
        resp.match_score = score;
        resp.projects_done = target_projects.len() as u32;
        out.push(resp);
    }
    Ok(out)
}

/// GET /api/v1/users?limit=20&skip=0&q=query
#[get("/?<limit>&<skip>&<q>")]
async fn list_users(
    limit: Option<i64>,
    skip: Option<u64>,
    q: Option<String>,
    auth: OptionalAuth,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let limit = limit.unwrap_or(20).min(100);
    let skip = skip.unwrap_or(0);

    let users: Vec<User> = if let Some(query) = q {
        state.users.search(&query).await?
    } else {
        state.users.list(limit, skip).await?
    };

    let viewer_id = auth.0.as_ref().map(|c| c.sub.as_str());
    let mut data = fill_match_scores(&**state, viewer_id, users).await?;
    // Sort by match score desc when viewer is authenticated.
    if viewer_id.is_some() {
        data.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    }

    let total = state.users.count().await?;

    Ok(Json(json!({
        "data": data,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

/// GET /api/v1/users/<id>
#[get("/<id>")]
async fn get_user(
    id: String,
    auth: OptionalAuth,
    state: &State<AppState>,
) -> ApiResult<UserResponse> {
    let user = state
        .users
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("User {} not found", id)))?;

    let viewer_id = auth.0.as_ref().map(|c| c.sub.as_str());
    let mut filled = fill_match_scores(&**state, viewer_id, vec![user]).await?;
    Ok(Json(filled.remove(0)))
}

/// PATCH /api/v1/users/<id>  (authenticated; can only update own profile)
#[patch("/<id>", data = "<req>")]
async fn update_user(
    id: String,
    mut req: Json<UpdateUserRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    if auth.0.sub != id && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    // Sanitize skill_tags + skill names against the catalog.
    if let Some(tags) = &req.skill_tags {
        req.skill_tags = Some(filter_valid_skills(tags));
    }
    if let Some(skills) = &req.skills {
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        let valid = filter_valid_skills(&names);
        req.skills = Some(
            skills
                .iter()
                .filter(|s| valid.iter().any(|v| v.eq_ignore_ascii_case(&s.name)))
                .cloned()
                .collect(),
        );
    }

    state.users.update(&id, &req).await?;

    Ok(Json(json!({ "success": true })))
}

/// Remove a user and all of the personal data tied to them. Side-effect cleanups
/// are best-effort (logged, never aborting) so a single failing collection can't
/// leave the account half-deleted; the user document is removed last.
async fn cascade_delete_user(state: &AppState, user_id: &str) -> Result<(), TeamderError> {
    // Projects the user leads — delete the project and its update feed.
    if let Ok(led) = state.projects.list_by_lead(user_id).await {
        for p in led {
            if let Err(e) = state.project_updates.delete_for_project(&p.id).await {
                tracing::warn!(project = %p.id, "failed to delete project updates: {e}");
            }
            if let Err(e) = state.projects.delete(&p.id).await {
                tracing::warn!(project = %p.id, "failed to delete project: {e}");
            }
        }
    }
    // Membership in projects led by others.
    if let Err(e) = state.projects.pull_member_everywhere(user_id).await {
        tracing::warn!("failed to remove user from project teams: {e}");
    }

    // Study groups the user created — delete; membership elsewhere — pull.
    if let Ok(created) = state.study_groups.list_by_creator(user_id).await {
        for g in created {
            if let Err(e) = state.study_groups.delete(&g.id).await {
                tracing::warn!(group = %g.id, "failed to delete study group: {e}");
            }
        }
    }
    if let Err(e) = state.study_groups.pull_member_everywhere(user_id).await {
        tracing::warn!("failed to remove user from study groups: {e}");
    }

    // Competition teams the user leads — delete; membership elsewhere — pull.
    if let Ok(teams) = state.competition_teams.list_for_user(user_id).await {
        for t in teams {
            if t.lead_user_id == user_id {
                if let Err(e) = state.competition_teams.delete(&t.id).await {
                    tracing::warn!(team = %t.id, "failed to delete competition team: {e}");
                }
            }
        }
    }
    if let Err(e) = state.competition_teams.pull_member_everywhere(user_id).await {
        tracing::warn!("failed to remove user from competition teams: {e}");
    }

    // Standalone personal records.
    if let Err(e) = state.invites.delete_for_user(user_id).await {
        tracing::warn!("failed to delete invites: {e}");
    }
    if let Err(e) = state.join_requests.delete_for_user(user_id).await {
        tracing::warn!("failed to delete join requests: {e}");
    }
    if let Err(e) = state.messages.delete_for_user(user_id).await {
        tracing::warn!("failed to delete messages: {e}");
    }
    if let Err(e) = state.notifications.delete_for_user(user_id).await {
        tracing::warn!("failed to delete notifications: {e}");
    }
    if let Err(e) = state.bookmarks.delete_for_user(user_id).await {
        tracing::warn!("failed to delete bookmarks: {e}");
    }
    if let Err(e) = state.peer_reviews.delete_for_user(user_id).await {
        tracing::warn!("failed to delete peer reviews: {e}");
    }

    // Uploaded files (avatar, banner, portfolio, resume, …) live under uploads/<user_id>/.
    let dir = std::path::Path::new("uploads").join(user_id);
    if dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&dir) {
            tracing::warn!("failed to remove uploads directory: {e}");
        }
    }

    // The account itself, last.
    state.users.delete(user_id).await
}

/// DELETE /api/v1/users/<id>  (admin, or own account) — cascades related data.
#[delete("/<id>")]
async fn delete_user(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    if auth.0.sub != id && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    cascade_delete_user(&**state, &id).await?;

    Ok(Json(json!({ "success": true })))
}

#[derive(Debug, serde::Deserialize)]
struct DeleteAccountRequest {
    /// Verification code emailed to the account holder (purpose "delete").
    code: String,
}

/// POST /api/v1/users/me/delete — self-service account closure.
/// Requires a fresh email verification code (request it via
/// `/auth/request-code` with purpose "delete"), then cascades all related data.
#[post("/me/delete", data = "<req>")]
async fn delete_account(
    req: Json<DeleteAccountRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let record = state
        .auth_codes
        .find(&user.email, "delete")
        .await?
        .ok_or(TeamderError::Unauthorized)?;
    if record.is_expired() || record.code != req.code.trim() {
        return Err(TeamderError::Unauthorized.into());
    }
    state.auth_codes.delete(&user.email, "delete").await?;

    cascade_delete_user(&**state, &auth.0.sub).await?;

    Ok(Json(json!({ "success": true })))
}

/// GET /api/v1/users/me
#[get("/me")]
async fn me(auth: AuthUser, state: &State<AppState>) -> ApiResult<UserResponse> {
    let user = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Current user not found".into()))?;
    if user.is_banned {
        return Err(TeamderError::Suspended("Your account has been suspended.".into()).into());
    }
    Ok(Json(user.into()))
}

#[derive(Debug, serde::Deserialize)]
struct SetPasswordRequest {
    new_password: String,
}

/// POST /api/v1/users/me/set-password — establish an initial password for an
/// account that doesn't have one yet (e.g. created via email verification).
/// Use change-password to rotate an existing password.
#[post("/me/set-password", data = "<req>")]
async fn set_password(
    req: Json<SetPasswordRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    if req.new_password.len() < 6 {
        return Err(TeamderError::Validation("Password must be at least 6 characters".into()).into());
    }
    let user = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
    if user.password_hash.is_some() {
        return Err(TeamderError::Conflict(
            "A password is already set — use change-password instead".into(),
        )
        .into());
    }
    let hash = bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;
    state.users.set_password_hash(&user.id, &hash).await?;
    Ok(Json(json!({ "success": true })))
}

#[derive(Debug, serde::Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

/// POST /api/v1/users/me/change-password — rotate an existing password.
#[post("/me/change-password", data = "<req>")]
async fn change_password(
    req: Json<ChangePasswordRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
    let hash = user.password_hash.as_deref().ok_or_else(|| {
        TeamderError::Validation("No password is set yet — use set-password instead".into())
    })?;
    let valid = bcrypt::verify(&req.old_password, hash)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;
    if !valid {
        return Err(TeamderError::Unauthorized.into());
    }
    if req.new_password.len() < 6 {
        return Err(TeamderError::Validation("New password must be at least 6 characters".into()).into());
    }
    let new_hash = bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;
    state.users.set_password_hash(&user.id, &new_hash).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/users/me/onboard
#[post("/me/onboard")]
async fn complete_onboarding(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let req = teamder_core::models::user::UpdateUserRequest {
        name: None, role: None, department: None, university: None, year: None, location: None,
        bio: None, skills: None, skill_tags: None, work_mode: None,
        availability: None, hours_per_week: None, languages: None,
        portfolio: None, avatar_url: None, banner_url: None, resume_url: None,
        onboarded: Some(true),
        headline: None, notify_email: None, notify_in_app: None, is_public: None,
        social_links: None, interests: None, timezone: None, goals: None,
        free_days: None,
    };
    state.users.update(&auth.0.sub, &req).await?;
    let _ = req;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_users, get_user, update_user, delete_user, delete_account, me, set_password, change_password, complete_onboarding]
}
