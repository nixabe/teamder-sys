use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::user::{UpdateUserRequest, UserResponse},
};

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// GET /api/v1/users?limit=20&skip=0&q=query
#[get("/?<limit>&<skip>&<q>")]
async fn list_users(
    limit: Option<i64>,
    skip: Option<u64>,
    q: Option<String>,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let limit = limit.unwrap_or(20).min(100);
    let skip = skip.unwrap_or(0);

    let users: Vec<UserResponse> = if let Some(query) = q {
        state
            .users
            .search(&query)
            .await?
            .into_iter()
            .map(UserResponse::from)
            .collect()
    } else {
        state
            .users
            .list(limit, skip)
            .await?
            .into_iter()
            .map(UserResponse::from)
            .collect()
    };

    let total = state.users.count().await?;

    Ok(Json(json!({
        "data": users,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

/// GET /api/v1/users/<id>
#[get("/<id>")]
async fn get_user(id: String, state: &State<AppState>) -> ApiResult<UserResponse> {
    let user = state
        .users
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("User {} not found", id)))?;
    Ok(Json(user.into()))
}

/// PATCH /api/v1/users/<id>  (authenticated; can only update own profile)
#[patch("/<id>", data = "<req>")]
async fn update_user(
    id: String,
    req: Json<UpdateUserRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    if auth.0.sub != id && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
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

pub fn routes() -> Vec<Route> {
    routes![list_users, get_user, update_user, delete_user, me]
}
