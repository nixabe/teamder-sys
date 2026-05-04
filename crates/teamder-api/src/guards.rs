use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};

use crate::{auth::Claims, state::AppState};

/// Request guard that extracts and verifies the JWT from the Authorization header.
pub struct AuthUser(pub Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = req.rocket().state::<AppState>().unwrap();

        let token = req
            .headers()
            .get_one("Authorization")
            .and_then(|v| v.strip_prefix("Bearer "));

        match token {
            None => Outcome::Error((Status::Unauthorized, ())),
            Some(t) => match crate::auth::verify_token(t, &state.jwt_secret) {
                Ok(claims) => Outcome::Success(AuthUser(claims)),
                Err(_) => Outcome::Error((Status::Unauthorized, ())),
            },
        }
    }
}

/// Optional authentication — succeeds with `Some(Claims)` when a valid token is
/// present, or `None` otherwise. Never fails the request.
pub struct OptionalAuth(pub Option<Claims>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalAuth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = req.rocket().state::<AppState>().unwrap();
        let token = req
            .headers()
            .get_one("Authorization")
            .and_then(|v| v.strip_prefix("Bearer "));
        let claims = token.and_then(|t| crate::auth::verify_token(t, &state.jwt_secret).ok());
        Outcome::Success(OptionalAuth(claims))
    }
}

/// Request guard that additionally requires the user to be an admin.
pub struct AdminUser(#[allow(dead_code)] pub Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthUser::from_request(req).await {
            Outcome::Success(AuthUser(claims)) if claims.is_admin => {
                Outcome::Success(AdminUser(claims))
            }
            Outcome::Success(_) => Outcome::Error((Status::Forbidden, ())),
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(f) => Outcome::Forward(f),
        }
    }
}
