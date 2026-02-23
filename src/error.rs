use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User already exists")]
    UserExists,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Insufficient shares")]
    InsufficientShares,

    #[error("Invalid stock symbol")]
    InvalidSymbol,

    #[error("Order not found")]
    OrderNotFound,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AppError::UserExists => (StatusCode::CONFLICT, "User already exists"),
            AppError::InsufficientBalance => (StatusCode::BAD_REQUEST, "Insufficient balance"),
            AppError::InsufficientShares => (StatusCode::BAD_REQUEST, "Insufficient shares"),
            AppError::InvalidSymbol => (StatusCode::BAD_REQUEST, "Invalid stock symbol"),
            AppError::OrderNotFound => (StatusCode::NOT_FOUND, "Order not found"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
