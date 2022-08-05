use axum::{http::StatusCode, response::IntoResponse};

pub async fn handler() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Not implemented")
}
