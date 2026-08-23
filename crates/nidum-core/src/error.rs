use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Standard HTTP error representation for Nidum applications.
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Validation Failed")]
    ValidationFailed(serde_json::Value),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("HTTP Error {0}: {1}")]
    Custom(StatusCode, String),
}

#[derive(Serialize)]
struct ErrorResponseBody {
    status_code: u16,
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<serde_json::Value>,
}

impl HttpError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            HttpError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            HttpError::Forbidden(_) => StatusCode::FORBIDDEN,
            HttpError::NotFound(_) => StatusCode::NOT_FOUND,
            HttpError::Conflict(_) => StatusCode::CONFLICT,
            HttpError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::Custom(code, _) => *code,
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let mut fields = None;
        let message = match &self {
            HttpError::BadRequest(m)
            | HttpError::Unauthorized(m)
            | HttpError::Forbidden(m)
            | HttpError::NotFound(m)
            | HttpError::Conflict(m)
            | HttpError::InternalServerError(m)
            | HttpError::Custom(_, m) => m.clone(),
            HttpError::ValidationFailed(f) => {
                fields = Some(f.clone());
                "Validation failed".to_string()
            }
        };

        let body = ErrorResponseBody {
            status_code: status.as_u16(),
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message,
            fields,
        };

        (status, Json(body)).into_response()
    }
}
