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

    let user = User::new(
        &req.email,
        password_hash,
        &req.name,
        &req.role,
        &req.department,
    );

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

pub fn routes() -> Vec<Route> {
    routes![register, login]
}
