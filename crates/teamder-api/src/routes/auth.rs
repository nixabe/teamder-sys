use chrono::Utc;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::create_token;
use crate::error::ApiError;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::user::{CreateUserRequest, User, UserResponse};

// ── Request / Response DTOs ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub success: bool,
    pub reset_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::post("/auth/register", data = "<body>")]
pub async fn register(
    state: &State<AppState>,
    body: Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let req = body.into_inner();

    // Validate
    if req.email.is_empty() || req.password.is_empty() || req.name.is_empty() {
        return Err(TeamderError::Validation("email, password, and name are required".into()).into());
    }

    // Check duplicate email
    if state.db.user_repo().find_by_email(&req.email).await?.is_some() {
        return Err(TeamderError::Conflict("Email already registered".into()).into());
    }

    // Hash password
    let password_hash =
        bcrypt::hash(&req.password, 12).map_err(|e| TeamderError::Internal(e.to_string()))?;

    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    // Build initials
    let initials: String = req
        .name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .collect();

    let user = User {
        id: id.clone(),
        email: req.email,
        password_hash,
        name: req.name,
        initials,
        role: req.role.unwrap_or_default(),
        department: req.department.unwrap_or_default(),
        university: req.university.unwrap_or_else(|| "Fu Jen Catholic University".to_string()),
        year: req.year.unwrap_or_else(|| "N/A".to_string()),
        location: None,
        bio: req.bio.unwrap_or_default(),
        skills: req.skills.unwrap_or_default(),
        skill_tags: req.skill_tags.unwrap_or_default(),
        gradient: String::new(),
        work_mode: None,
        availability: None,
        hours_per_week: None,
        languages: req
            .languages
            .unwrap_or_else(|| vec!["Chinese".into(), "English".into()]),
        portfolio: vec![],
        reviews: vec![],
        match_score: None,
        rating: 0.0,
        projects_done: 0,
        collaborations: 0,
        avatar_url: None,
        resume_url: None,
        reset_token: None,
        reset_token_expires_at: None,
        is_admin: false,
        is_publisher: false,
        is_public: true,
        onboarded: false,
        headline: None,
        notify_email: true,
        notify_in_app: true,
        social_links: vec![],
        interests: req.interests.unwrap_or_default(),
        timezone: None,
        goals: None,
        free_days: vec![],
        created_at: now,
        updated_at: now,
    };

    state.db.user_repo().create(&user).await?;

    let token =
        create_token(&id, &state.jwt_secret).map_err(|e| TeamderError::Internal(e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

#[rocket::post("/auth/login", data = "<body>")]
pub async fn login(
    state: &State<AppState>,
    body: Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let req = body.into_inner();

    let user = state
        .db
        .user_repo()
        .find_by_email(&req.email)
        .await?
        .ok_or_else(|| TeamderError::Unauthorized("Invalid email or password".into()))?;

    let valid = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    if !valid {
        return Err(TeamderError::Unauthorized("Invalid email or password".into()).into());
    }

    let token = create_token(&user.id, &state.jwt_secret)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

#[rocket::post("/auth/forgot-password", data = "<body>")]
pub async fn forgot_password(
    state: &State<AppState>,
    body: Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, ApiError> {
    let req = body.into_inner();

    let user = state.db.user_repo().find_by_email(&req.email).await?;
    if user.is_none() {
        // Don't reveal whether email exists, but in dev mode return success
        return Ok(Json(ForgotPasswordResponse {
            success: true,
            reset_token: String::new(),
        }));
    }

    // Generate 64-char hex token
    let reset_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let expires_at = Utc::now() + chrono::Duration::minutes(30);

    state
        .db
        .user_repo()
        .set_reset_token(&req.email, &reset_token, expires_at)
        .await?;

    Ok(Json(ForgotPasswordResponse {
        success: true,
        reset_token,
    }))
}

#[rocket::post("/auth/reset-password", data = "<body>")]
pub async fn reset_password(
    state: &State<AppState>,
    body: Json<ResetPasswordRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let req = body.into_inner();

    let user = state
        .db
        .user_repo()
        .find_by_reset_token(&req.token)
        .await?
        .ok_or_else(|| TeamderError::Validation("Invalid or expired reset token".into()))?;

    // Check expiry
    if let Some(expires) = user.reset_token_expires_at {
        if Utc::now() > expires {
            return Err(TeamderError::Validation("Reset token has expired".into()).into());
        }
    } else {
        return Err(TeamderError::Validation("Invalid reset token".into()).into());
    }

    // Hash new password
    let password_hash =
        bcrypt::hash(&req.password, 12).map_err(|e| TeamderError::Internal(e.to_string()))?;

    use mongodb::bson;
    let update = bson::doc! {
        "password_hash": &password_hash,
        "updated_at": bson::DateTime::from_chrono(Utc::now()),
    };
    state.db.user_repo().update(&user.id, update).await?;
    state.db.user_repo().clear_reset_token(&user.id).await?;

    Ok(Json(SuccessResponse { success: true }))
}
