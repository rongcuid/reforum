use eyre::*;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
};
use maud::html;
use nanoid::nanoid;
use sea_orm::prelude::DateTimeUtc;
use secrecy::{Secret, SecretString};
use tracing::instrument;

use crate::core::session::{insert_session, Session};

#[instrument]
pub async fn handler(
    session: Session,
    Extension(db): Extension<sea_orm::DatabaseConnection>,
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
