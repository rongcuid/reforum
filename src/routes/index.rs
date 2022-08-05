use axum::{http::StatusCode, response::{IntoResponse, Html}};
use maud::{html, Render};

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
