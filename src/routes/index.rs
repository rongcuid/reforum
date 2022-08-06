use deadpool_sqlite::Pool;
use eyre::*;

use axum::{
    http::StatusCode,
    response::{Html},
    Extension,
};
use maud::html;


use tracing::{instrument};

use crate::core::session::{verify_session, Session};

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
                    p{"Hello, "(format!("user {:?}", data))"!"}
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
