use axum::{http::StatusCode, response::IntoResponse};
use tracing::instrument;

#[instrument]
pub async fn handler() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Not implemented")
}
