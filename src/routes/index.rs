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
use tracing::{error, instrument};

use crate::core::session::{self, verify_session, Session};

#[instrument(skip_all)]
pub async fn handler(
    session: Session,
    Extension(db): Extension<Pool>,
) -> Result<Html<String>, StatusCode> {
    if let Some(data) = session.get() {
        if verify_session(&db.get().await.unwrap(), &session)
            .await
            .unwrap()
        {
            return Ok(Html(
                html! {
                    h1{"Index of Reforum"}
                    p{"Hello, "(format!("user {}", data.user_id))"!"}
                    a href="/logout" { "Logout" }
                }
                .0,
            ));
        }
    }

    Ok(Html(
        html! {
            h1{"Index of Reforum"}
            p{"Hello, Anonymous!"}
            a href="/login" { "Login" }
        }
        .0,
    ))
}
