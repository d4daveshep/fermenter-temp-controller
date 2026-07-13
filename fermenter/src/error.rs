use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to parse reading: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("serial I/O error: {0}")]
    Serial(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("time-series storage error: {0}")]
    Store(String),

    #[error("template render error: {0}")]
    Render(String),
}

/// Maps `AppError` into an HTTP response for Axum handlers. Only `Render`
/// errors are expected to surface through the web layer today (handlers
/// don't yet call parse/serial/config/store fallibly), but every variant
/// gets a sensible response rather than failing to compile a catch-all.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Parse(_) => StatusCode::BAD_REQUEST,
            AppError::Serial(_) | AppError::Config(_) | AppError::Store(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, self.to_string()).into_response()
    }
}
