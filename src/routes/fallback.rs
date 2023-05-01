use poem::{handler, http::StatusCode, IntoResponse};
use poem::error::NotFoundError;

pub async fn handler_404(_: NotFoundError) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not found")
}
