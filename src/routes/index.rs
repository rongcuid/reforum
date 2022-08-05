use deadpool_sqlite::Pool;
use eyre::*;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
};
use maud::html;
use nanoid::nanoid;
use secrecy::{Secret, SecretString};
use tracing::instrument;

use crate::core::session::{ Session};

#[instrument(skip_all)]
pub async fn handler(
    session: Session,
    Extension(db): Extension<Pool>,
) -> Result<Html<String>, StatusCode> {
    let id_str = if let Some(data) = session.get() {
        format!("user {}", data.user_id)
    } else {
        "Anonymous".to_owned()
    };
    Ok(Html(
        html! {h1{"Index of Reforum"}p{"Hello, "(id_str)"!"}}.0,
    ))
}
