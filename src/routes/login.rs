
use axum::{http::StatusCode, response::IntoResponse};
use maud::html;

pub async fn handler() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Not implemented")
}
