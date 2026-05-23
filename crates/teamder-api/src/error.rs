use rocket::http::Status;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Request;
use serde::Serialize;
use teamder_core::error::TeamderError;

/// JSON error body returned to the client.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// JSON error envelope.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

/// Wrapper so we can implement Rocket's `Responder` for `TeamderError`.
pub struct ApiError(pub TeamderError);

impl From<TeamderError> for ApiError {
    fn from(err: TeamderError) -> Self {
        ApiError(err)
    }
}

impl ApiError {
    fn code_string(&self) -> String {
        match &self.0 {
            TeamderError::NotFound(_) => "NOT_FOUND".to_string(),
            TeamderError::Unauthorized(_) => "UNAUTHORIZED".to_string(),
            TeamderError::Forbidden(_) => "FORBIDDEN".to_string(),
            TeamderError::Validation(_) => "VALIDATION_ERROR".to_string(),
            TeamderError::Conflict(_) => "CONFLICT".to_string(),
            TeamderError::Database(_) => "DATABASE_ERROR".to_string(),
            TeamderError::Internal(_) => "INTERNAL_ERROR".to_string(),
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let status =
            Status::from_code(self.0.status_code()).unwrap_or(Status::InternalServerError);

        let body = ErrorResponse {
            error: ErrorBody {
                code: self.code_string(),
                message: self.0.message().to_string(),
            },
        };

        (status, Json(body)).respond_to(req)
    }
}
