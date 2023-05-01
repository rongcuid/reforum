// use axum::{http::StatusCode, response::Html, Extension};
use maud::html;
use poem::{handler, http::StatusCode};
use poem::web::Html;

// use crate::auth::extractor::UserAuth;
use tracing::instrument;

#[instrument(skip_all)]
#[handler]
pub async fn handler() -> Result<Html<String>, StatusCode> {
    Ok(Html(
        html! {
                h1{"Index of Reforum"}
                p{"Hello, Anonymous!"}
                a href="/login" { "Login" }
            }
            .0,
    ))
    // if let Some(auth) = auth {
    //     Ok(Html(
    //         html! {
    //             h1{"Index of Reforum"}
    //             p{"Hello, "(format!("user {:?}", auth))"!"}
    //             a href="/logout" { "Logout" }
    //         }
    //         .0,
    //     ))
    // } else {
    //     Ok(Html(
    //         html! {
    //             h1{"Index of Reforum"}
    //             p{"Hello, Anonymous!"}
    //             a href="/login" { "Login" }
    //         }
    //         .0,
    //     ))
    // }
}
