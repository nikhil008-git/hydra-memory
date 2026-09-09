use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

pub enum AppError {
    BadRequest(String),
    BadGateway(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, message)
            }
            AppError::BadGateway(message) => {
                (StatusCode::BAD_GATEWAY, message)
            }
        };

        (
            status,
            Json(json!({
                "error": message
            })),
        )
            .into_response()
    }
}