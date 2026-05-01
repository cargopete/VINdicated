use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid VIN: {0}")]
    InvalidVin(String),
    #[error("Not found")]
    NotFound,
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::InvalidVin(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found".into()),
            ApiError::Cache(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Not found")]
    NotFound,
    #[error("Rate limited")]
    RateLimited,
    #[error("Not supported")]
    NotSupported,
    #[error("Unavailable: {0}")]
    Unavailable(String),
}
