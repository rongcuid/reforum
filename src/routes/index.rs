
use axum::{http::StatusCode, response::IntoResponse};
use maud::html;

pub async fn handler() -> impl IntoResponse {
    (StatusCode::OK, html! {
        h1 { "Index of Reforum" }
        p { "Hello, world!" }
    }.0)
}
