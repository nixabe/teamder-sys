use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
    serde::json::Json,
};
use serde_json::json;
use teamder_core::error::TeamderError;

/// Wraps `TeamderError` so Rocket can respond with JSON error bodies.
#[derive(Debug)]
pub struct ApiError(pub TeamderError);

impl From<TeamderError> for ApiError {
    fn from(e: TeamderError) -> Self {
        ApiError(e)
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        let (status, code) = match &self.0 {
            TeamderError::NotFound(_) => (Status::NotFound, "not_found"),
            TeamderError::Unauthorized => (Status::Unauthorized, "unauthorized"),
            TeamderError::Forbidden => (Status::Forbidden, "forbidden"),
            TeamderError::Validation(_) => (Status::UnprocessableEntity, "validation_error"),
            TeamderError::Conflict(_) => (Status::Conflict, "conflict"),
            TeamderError::Database(_) | TeamderError::Internal(_) => {
                (Status::InternalServerError, "internal_error")
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": self.0.to_string(),
            }
        });

        Response::build_from(Json(body).respond_to(req)?)
            .status(status)
            .ok()
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;
