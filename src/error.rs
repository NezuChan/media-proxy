use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),

    #[error("{message}")]
    BadRequest { status: StatusCode, message: String },

    #[error("{0}")]
    Crypto(String),

    #[error("failed to fetch media: {0}")]
    Fetch(#[from] reqwest::Error),

    #[error("origin returned status {status}")]
    Upstream { status: u16 },

    #[error("failed to process image: {0}")]
    Image(String),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::BadRequest { status, .. } => *status,
            AppError::Crypto(_) => StatusCode::BAD_REQUEST,
            AppError::Upstream { .. } => StatusCode::BAD_GATEWAY,
            AppError::Config(_)
            | AppError::Fetch(_)
            | AppError::Image(_)
            | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::BadRequest { message, .. } => message.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    #[serde(rename = "statusCode")]
    status_code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorBody {
            status_code: status.as_u16(),
            message: self.message(),
        };
        (status, Json(body)).into_response()
    }
}
