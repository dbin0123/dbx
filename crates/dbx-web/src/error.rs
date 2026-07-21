use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

pub struct AppError(pub String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AppError {
    pub fn internal(msg: impl Into<String>) -> Self {
        AppError(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError(s.to_string())
    }
}
