use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};
use maud::html;
use tracing::instrument;

use crate::core::session::Session;

#[instrument]
pub async fn handler(session: Session) -> impl IntoResponse {
    let id_str = if let Some(data) = session.get() {
        format!("user {}", data.user_id)
    } else {
        "Anonymous".to_owned()
    };
    (
        StatusCode::OK,
        Html(
            html! {
                h1 { "Index of Reforum" }
                p { "Hello, " (id_str) "!" }
            }
            .0,
        ),
    )
}
