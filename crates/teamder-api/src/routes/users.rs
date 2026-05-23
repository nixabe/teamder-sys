use chrono::Utc;
use mongodb::bson;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::guards::{AuthUser, OptionalAuth};
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::user::{UpdateUserRequest, UserResponse};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PaginatedUsers {
    pub users: Vec<UserResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/users?<page>&<limit>&<q>")]
pub async fn list_users(
    state: &State<AppState>,
    page: Option<u64>,
    limit: Option<i64>,
    q: Option<String>,
) -> Result<Json<PaginatedUsers>, ApiError> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);
    let skip = (page.saturating_sub(1)) * (limit as u64);

    let (users, total) = state
        .db
        .user_repo()
        .list(skip, limit, q.as_deref())
        .await?;

    let users: Vec<UserResponse> = users.into_iter().map(Into::into).collect();

    Ok(Json(PaginatedUsers {
        users,
        total,
        page,
        limit,
    }))
}

#[rocket::get("/users/me")]
pub async fn get_me(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    Ok(Json(user.into()))
}

#[rocket::get("/users/<id>")]
pub async fn get_user(
    state: &State<AppState>,
    id: &str,
    viewer: OptionalAuth,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state
        .db
        .user_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let mut resp: UserResponse = user.into();

    // Compute match_score if viewer is authenticated
    if let Some(viewer_id) = &viewer.0 {
        if viewer_id != id {
            if let Ok(Some(viewer_user)) = state.db.user_repo().find_by_id(viewer_id).await {
                let viewer_tags: std::collections::HashSet<&str> =
                    viewer_user.skill_tags.iter().map(|s| s.as_str()).collect();
                let target_tags: std::collections::HashSet<&str> =
                    resp.skill_tags.iter().map(|s| s.as_str()).collect();

                if !viewer_tags.is_empty() && !target_tags.is_empty() {
                    let intersection = viewer_tags.intersection(&target_tags).count();
                    let union = viewer_tags.union(&target_tags).count();
                    let score = ((intersection as f64 / union as f64) * 100.0) as u8;
                    resp.match_score = Some(score);
                }
            }
        }
    }

    Ok(Json(resp))
}

#[rocket::patch("/users/<id>", data = "<body>")]
pub async fn update_user(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // Check permission: own profile or admin
    if auth.user_id != id {
        let caller = state
            .db
            .user_repo()
            .find_by_id(&auth.user_id)
            .await?
            .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
        if !caller.is_admin {
            return Err(TeamderError::Forbidden("Cannot edit another user's profile".into()).into());
        }
    }

    let req = body.into_inner();
    let mut update = bson::doc! {};

    if let Some(v) = &req.name {
        update.insert("name", v.as_str());
    }
    if let Some(v) = &req.initials {
        update.insert("initials", v.as_str());
    }
    if let Some(v) = &req.role {
        update.insert("role", v.as_str());
    }
    if let Some(v) = &req.department {
        update.insert("department", v.as_str());
    }
    if let Some(v) = &req.university {
        update.insert("university", v.as_str());
    }
    if let Some(v) = &req.year {
        update.insert("year", v.as_str());
    }
    if let Some(v) = &req.location {
        update.insert("location", v.as_str());
    }
    if let Some(v) = &req.bio {
        update.insert(
            "bio",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(v) = &req.skills {
        update.insert(
            "skills",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(v) = &req.skill_tags {
        update.insert(
            "skill_tags",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(v) = &req.gradient {
        update.insert("gradient", v.as_str());
    }
    if let Some(v) = &req.work_mode {
        update.insert("work_mode", v.as_str());
    }
    if let Some(v) = &req.availability {
        update.insert("availability", v.as_str());
    }
    if let Some(v) = &req.hours_per_week {
        update.insert("hours_per_week", v.as_str());
    }
    if let Some(v) = &req.languages {
        update.insert(
            "languages",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(v) = &req.portfolio {
        update.insert(
            "portfolio",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(v) = &req.avatar_url {
        update.insert("avatar_url", v.as_str());
    }
    if let Some(v) = &req.resume_url {
        update.insert("resume_url", v.as_str());
    }
    if let Some(v) = req.is_public {
        update.insert("is_public", v);
    }
    if let Some(v) = req.onboarded {
        update.insert("onboarded", v);
    }
    if let Some(v) = &req.headline {
        update.insert("headline", v.as_str());
    }
    if let Some(v) = req.notify_email {
        update.insert("notify_email", v);
    }
    if let Some(v) = req.notify_in_app {
        update.insert("notify_in_app", v);
    }
    if let Some(v) = &req.social_links {
        update.insert(
            "social_links",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(v) = &req.interests {
        update.insert(
            "interests",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(v) = &req.timezone {
        update.insert("timezone", v.as_str());
    }
    if let Some(v) = &req.goals {
        update.insert("goals", v.as_str());
    }
    if let Some(v) = &req.free_days {
        update.insert(
            "free_days",
            bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?,
        );
    }
    if let Some(pw) = &req.password {
        let hash =
            bcrypt::hash(pw, 12).map_err(|e| TeamderError::Internal(e.to_string()))?;
        update.insert("password_hash", hash);
    }

    update.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));

    state.db.user_repo().update(id, update).await?;

    let updated = state
        .db
        .user_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    Ok(Json(updated.into()))
}

#[rocket::delete("/users/<id>")]
pub async fn delete_user(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    if auth.user_id != id {
        let caller = state
            .db
            .user_repo()
            .find_by_id(&auth.user_id)
            .await?
            .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
        if !caller.is_admin {
            return Err(TeamderError::Forbidden("Cannot delete another user".into()).into());
        }
    }

    state.db.user_repo().delete(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/users/me/change-password", data = "<body>")]
pub async fn change_password(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<ChangePasswordRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let req = body.into_inner();

    let user = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let valid = bcrypt::verify(&req.old_password, &user.password_hash)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    if !valid {
        return Err(TeamderError::Validation("Current password is incorrect".into()).into());
    }

    let new_hash =
        bcrypt::hash(&req.new_password, 12).map_err(|e| TeamderError::Internal(e.to_string()))?;

    let update = bson::doc! {
        "password_hash": &new_hash,
        "updated_at": bson::DateTime::from_chrono(Utc::now()),
    };

    state.db.user_repo().update(&auth.user_id, update).await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/users/me/onboard")]
pub async fn onboard(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<SuccessResponse>, ApiError> {
    state.db.user_repo().mark_onboarded(&auth.user_id).await?;
    Ok(Json(SuccessResponse { success: true }))
}
