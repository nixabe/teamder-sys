use thiserror::Error;

/// Central error type for the Teamder domain.
///
/// Each variant carries a human-readable message and maps to an HTTP status code.
#[derive(Debug, Error)]
pub enum TeamderError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl TeamderError {
    /// Returns the HTTP status code corresponding to this error variant.
    pub fn status_code(&self) -> u16 {
        match self {
            TeamderError::NotFound(_) => 404,
            TeamderError::Unauthorized(_) => 401,
            TeamderError::Forbidden(_) => 403,
            TeamderError::Validation(_) => 422,
            TeamderError::Conflict(_) => 409,
            TeamderError::Database(_) => 500,
            TeamderError::Internal(_) => 500,
        }
    }

    /// Returns the error message string.
    pub fn message(&self) -> &str {
        match self {
            TeamderError::NotFound(msg)
            | TeamderError::Unauthorized(msg)
            | TeamderError::Forbidden(msg)
            | TeamderError::Validation(msg)
            | TeamderError::Conflict(msg)
            | TeamderError::Database(msg)
            | TeamderError::Internal(msg) => msg,
        }
    }
}

// ── Conversions ──────────────────────────────────────────────────────────────

impl From<anyhow::Error> for TeamderError {
    fn from(err: anyhow::Error) -> Self {
        TeamderError::Internal(err.to_string())
    }
}

/// Error response body returned by the API.
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
}

impl From<&TeamderError> for ErrorResponse {
    fn from(err: &TeamderError) -> Self {
        Self {
            error: err.to_string(),
            status: err.status_code(),
        }
    }
}
