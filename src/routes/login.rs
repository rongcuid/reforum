use thiserror::*;

use poem::*;

use maud::html;
use poem::error::ResponseError;
use poem::http::{header, StatusCode};
use poem::session::Session;
use poem::web::{Data, Form, Html};

use crate::auth::authentication::LoginCredential;
use crate::configuration::SQLite3Settings;
use tracing::instrument;

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("forbidden")]
    AlreadyLoggedIn,
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal")]
    InternalError,
}

impl ResponseError for LoginError {
    fn status(&self) -> StatusCode {
        match self {
            LoginError::AlreadyLoggedIn => StatusCode::FORBIDDEN,
            LoginError::Unauthorized => StatusCode::UNAUTHORIZED,
            LoginError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[instrument(skip_all)]
#[handler]
pub async fn get_handler(session: &Session) -> Result<impl IntoResponse, LoginError> {
    if session.get::<i64>("uid").is_some() {
        return Err(LoginError::AlreadyLoggedIn);
    }
    Ok(Html(
        html! {
            h1{"Login"}
            form method="post" {
                div {
                    label for="username" { "Username" }
                    input type="text" name="username";
                }
                div {
                    label for="password" { "Password" }
                    input type="password" name="password";
                }
                button type="submit" { "Login" }
            }
        }
        .0,
    ))
}

#[instrument(skip_all, fields(username=cred.username))]
#[handler]
pub async fn post_handler(
    Form(cred): Form<LoginCredential>,
    session: &Session,
    db: Data<&SQLite3Settings>,
) -> Result<impl IntoResponse, LoginError> {
    let user_id = cred
        .validate(&db)
        .await
        .map_err(|_| LoginError::InternalError)?;
    if let Some(user_id) = user_id {
        session.set("uid", user_id);
        Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/")
            .finish())
    } else {
        Err(LoginError::Unauthorized)
    }
}
