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
}
