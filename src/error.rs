use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebDavError {
    #[error("Resource not found: {0}")]
    NotFound(PathBuf),

    #[error("Resource already exists: {0}")]
    AlreadyExists(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Lock conflict")]
    LockConflict,

    #[error("XML error: {0}")]
    XmlError(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

impl WebDavError {
    pub fn status_code(&self) -> http::StatusCode {
        use http::StatusCode;
        match self {
            WebDavError::NotFound(_) => StatusCode::NOT_FOUND,
            WebDavError::AlreadyExists(_) => StatusCode::CONFLICT,
            WebDavError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            WebDavError::LockConflict => StatusCode::LOCKED,
            WebDavError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
