use axum::{http::StatusCode, response::Html, Extension};
use maud::html;

use crate::auth::extractor::UserAuth;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn handler(auth: Option<UserAuth>) -> Result<Html<String>, StatusCode> {
    if let Some(auth) = auth {
        Ok(Html(
            html! {
                h1{"Index of Reforum"}
                p{"Hello, "(format!("user {:?}", auth))"!"}
                a href="/logout" { "Logout" }
            }
            .0,
        ))
    } else {
        Ok(Html(
            html! {
                h1{"Index of Reforum"}
                p{"Hello, Anonymous!"}
                a href="/login" { "Login" }
            }
            .0,
        ))
    }
}
