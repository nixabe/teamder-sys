use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::auth::verify_token;
use crate::state::AppState;

// ── AuthUser ────────────────────────────────────────────────────────────────

/// Extracts the authenticated user_id from `Authorization: Bearer <token>`.
pub struct AuthUser {
    pub user_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = match req.rocket().state::<AppState>() {
            Some(s) => s,
            None => return Outcome::Error((Status::InternalServerError, "Missing AppState")),
        };

        let header = match req.headers().get_one("Authorization") {
            Some(h) => h,
            None => return Outcome::Error((Status::Unauthorized, "Missing Authorization header")),
        };

        let token = header.strip_prefix("Bearer ").unwrap_or(header);

        match verify_token(token, &state.jwt_secret) {
            Ok(claims) => Outcome::Success(AuthUser {
                user_id: claims.sub,
            }),
            Err(_) => Outcome::Error((Status::Unauthorized, "Invalid or expired token")),
        }
    }
}

// ── OptionalAuth ────────────────────────────────────────────────────────────

/// Same as AuthUser, but returns `None` instead of erroring if no token.
pub struct OptionalAuth(pub Option<String>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalAuth {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = match req.rocket().state::<AppState>() {
            Some(s) => s,
            None => return Outcome::Success(OptionalAuth(None)),
        };

        let header = match req.headers().get_one("Authorization") {
            Some(h) => h,
            None => return Outcome::Success(OptionalAuth(None)),
        };

        let token = header.strip_prefix("Bearer ").unwrap_or(header);

        match verify_token(token, &state.jwt_secret) {
            Ok(claims) => Outcome::Success(OptionalAuth(Some(claims.sub))),
            Err(_) => Outcome::Success(OptionalAuth(None)),
        }
    }
}

// ── AdminUser ───────────────────────────────────────────────────────────────

/// AuthUser + checks `is_admin` on the user document.
pub struct AdminUser {
    pub user_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = match req.rocket().state::<AppState>() {
            Some(s) => s,
            None => return Outcome::Error((Status::InternalServerError, "Missing AppState")),
        };

        let header = match req.headers().get_one("Authorization") {
            Some(h) => h,
            None => return Outcome::Error((Status::Unauthorized, "Missing Authorization header")),
        };

        let token = header.strip_prefix("Bearer ").unwrap_or(header);

        let claims = match verify_token(token, &state.jwt_secret) {
            Ok(c) => c,
            Err(_) => return Outcome::Error((Status::Unauthorized, "Invalid or expired token")),
        };

        // Check admin flag in database
        let user = match state.db.user_repo().find_by_id(&claims.sub).await {
            Ok(Some(u)) => u,
            _ => return Outcome::Error((Status::Unauthorized, "User not found")),
        };

        if !user.is_admin {
            return Outcome::Error((Status::Forbidden, "Admin access required"));
        }

        Outcome::Success(AdminUser {
            user_id: claims.sub,
        })
    }
}

// ── PublisherUser ───────────────────────────────────────────────────────────

/// AuthUser + checks `is_publisher` OR `is_admin` on the user document.
pub struct PublisherUser {
    pub user_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PublisherUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = match req.rocket().state::<AppState>() {
            Some(s) => s,
            None => return Outcome::Error((Status::InternalServerError, "Missing AppState")),
        };

        let header = match req.headers().get_one("Authorization") {
            Some(h) => h,
            None => return Outcome::Error((Status::Unauthorized, "Missing Authorization header")),
        };

        let token = header.strip_prefix("Bearer ").unwrap_or(header);

        let claims = match verify_token(token, &state.jwt_secret) {
            Ok(c) => c,
            Err(_) => return Outcome::Error((Status::Unauthorized, "Invalid or expired token")),
        };

        let user = match state.db.user_repo().find_by_id(&claims.sub).await {
            Ok(Some(u)) => u,
            _ => return Outcome::Error((Status::Unauthorized, "User not found")),
        };

        if !user.is_publisher && !user.is_admin {
            return Outcome::Error((Status::Forbidden, "Publisher access required"));
        }

        Outcome::Success(PublisherUser {
            user_id: claims.sub,
        })
    }
}
