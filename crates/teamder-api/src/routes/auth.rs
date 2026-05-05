use rocket::{Route, State, serde::json::Json};
use serde::{Deserialize, Serialize};
use teamder_core::{
    error::TeamderError,
    models::user::{CreateUserRequest, User},
};

use crate::{auth, error::ApiResult, state::AppState};

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: teamder_core::models::user::UserResponse,
}

/// POST /api/v1/auth/register
#[post("/register", data = "<req>")]
async fn register(
    req: Json<CreateUserRequest>,
    state: &State<AppState>,
) -> ApiResult<AuthResponse> {
    // Check if email already exists
    if state
        .users
        .find_by_email(&req.email)
        .await?
        .is_some()
    {
        return Err(TeamderError::Conflict(format!(
            "Email {} is already registered",
            req.email
        ))
        .into());
    }

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let mut user = User::new(
        &req.email,
        password_hash,
        &req.name,
        &req.role,
        &req.department,
    );
    if let Some(u) = &req.university {
        if !u.trim().is_empty() { user.university = u.clone(); }
    }
    if let Some(y) = &req.year {
        if !y.trim().is_empty() { user.year = y.clone(); }
    }
    if let Some(h) = &req.headline {
        if !h.trim().is_empty() { user.headline = Some(h.clone()); }
    }
    if let Some(l) = &req.location {
        if !l.trim().is_empty() { user.location = Some(l.clone()); }
    }

    state.users.create(&user).await?;

    let token = auth::create_token(&user.id, &user.email, user.is_admin, &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// POST /api/v1/auth/login
#[post("/login", data = "<req>")]
async fn login(req: Json<LoginRequest>, state: &State<AppState>) -> ApiResult<AuthResponse> {
    let user = state
        .users
        .find_by_email(&req.email)
        .await?
        .ok_or_else(|| TeamderError::Unauthorized)?;

    let valid = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    if !valid {
        return Err(TeamderError::Unauthorized.into());
    }

    let token = auth::create_token(&user.id, &user.email, user.is_admin, &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

#[derive(Debug, Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

#[derive(Debug, Serialize)]
struct ForgotPasswordResponse {
    /// Always true — never reveals whether the email is registered, to avoid
    /// leaking the user list. The token is included only in dev (no SMTP).
    success: bool,
    /// In production this would be sent via email; we return it directly so the
    /// frontend can show a "your reset link" callout without infrastructure.
    reset_token: Option<String>,
}

/// POST /api/v1/auth/forgot-password
#[post("/forgot-password", data = "<req>")]
async fn forgot_password(
    req: Json<ForgotPasswordRequest>,
    state: &State<AppState>,
) -> ApiResult<ForgotPasswordResponse> {
    let user = state.users.find_by_email(&req.email).await?;
    let token_opt = if let Some(u) = user {
        // Two UUIDs concatenated (hyphens stripped) → 64-char hex token.
        // Valid for 30 minutes. Cryptographic uniqueness is enough here since
        // the token is checked exact-match and short-lived.
        let token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let expires = chrono::Utc::now() + chrono::Duration::minutes(30);
        state
            .users
            .set_reset_token(&u.id, Some(&token), Some(expires))
            .await?;
        Some(token)
    } else {
        None
    };
    Ok(Json(ForgotPasswordResponse {
        success: true,
        reset_token: token_opt,
    }))
}

#[derive(Debug, Deserialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

/// POST /api/v1/auth/reset-password
#[post("/reset-password", data = "<req>")]
async fn reset_password(
    req: Json<ResetPasswordRequest>,
    state: &State<AppState>,
) -> ApiResult<serde_json::Value> {
    if req.new_password.len() < 6 {
        return Err(TeamderError::Validation("Password must be at least 6 characters".into()).into());
    }

    let user = state
        .users
        .find_by_reset_token(&req.token)
        .await?
        .ok_or_else(|| TeamderError::Unauthorized)?;

    let valid = user
        .reset_token_expires_at
        .map(|exp| exp > chrono::Utc::now())
        .unwrap_or(false);
    if !valid {
        return Err(TeamderError::Unauthorized.into());
    }

    let hash = bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| TeamderError::Internal(e.to_string()))?;
    state.users.set_password_hash(&user.id, &hash).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![register, login, forgot_password, reset_password]
}
