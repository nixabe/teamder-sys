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

/// DELETE /api/v1/users/<id>  (own account or admin)
#[delete("/<id>")]
async fn delete_user(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    if auth.0.sub != id && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    state.users.delete(&id).await?;

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
    Ok(Json(user.into()))
}

#[derive(Debug, serde::Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

/// POST /api/v1/users/me/change-password
#[post("/me/change-password", data = "<req>")]
async fn change_password(
    req: Json<ChangePasswordRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
    let valid = bcrypt::verify(&req.old_password, &user.password_hash)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;
    if !valid {
        return Err(TeamderError::Unauthorized.into());
    }
    if req.new_password.len() < 6 {
        return Err(TeamderError::Validation("New password must be ≥ 6 characters".into()).into());
    }
    let hash = bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;
    state.users.set_password_hash(&user.id, &hash).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/users/me/onboard
#[post("/me/onboard")]
async fn complete_onboarding(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let req = teamder_core::models::user::UpdateUserRequest {
        name: None, role: None, department: None, year: None, location: None,
        bio: None, skills: None, skill_tags: None, work_mode: None,
        availability: None, hours_per_week: None, languages: None,
        portfolio: None, resume_url: None,
        onboarded: Some(true),
        headline: None, notify_email: None, notify_in_app: None, is_public: None,
    };
    state.users.update(&auth.0.sub, &req).await?;
    let _ = req;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_users, get_user, update_user, delete_user, me, change_password, complete_onboarding]
}
