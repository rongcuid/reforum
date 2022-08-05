use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};
use maud::html;
use tracing::instrument;

#[instrument]
pub async fn handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Html(
            html! {
                h1 { "Index of Reforum" }
                p { "Hello, world!" }
            }
            .0,
        ),
    )
}
