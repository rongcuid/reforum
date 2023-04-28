use axum::response::Html;
use axum::{http::StatusCode, response::IntoResponse};

pub async fn handler_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 not found")
}
