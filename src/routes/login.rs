use thiserror::*;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Extension, Form,
};
use axum_extra::extract::SignedCookieJar;
use cookie::{Cookie, SameSite};
use deadpool_sqlite::Pool;
use maud::html;

use time::{Duration, OffsetDateTime};
use tracing::instrument;

use crate::{
    core::{
        authentication::LoginCredential,
        session::{Session, SessionError},
    },
    startup::SessionCookieName,
};

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("forbidden")]
    AlreadyLoggedIn,
    #[error("unauthorized")]
    Unauthorized,
    #[error(transparent)]
    SessionError(#[from] SessionError),
    #[error(transparent)]
    Other(#[from] eyre::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        match self {
            LoginError::AlreadyLoggedIn => {
                (StatusCode::FORBIDDEN, "Already logged in").into_response()
            }
            LoginError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Incorrect username or password").into_response()
            }
            LoginError::SessionError(err) => err.into_response(),
            LoginError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            )
                .into_response(),
        }
    }
}

async fn new_session_to_cookie(
    session: &mut Session,
    session_name: &str,
    jar: SignedCookieJar,
    user_id: i64,
) -> Result<SignedCookieJar, LoginError> {
    let expires_at = Some(OffsetDateTime::now_utc() + Duration::weeks(4));
    let session = session.insert(user_id, &expires_at).await?;
    let jar = if let Ok(s) = serde_json::to_string(&session) {
        jar.add(
            Cookie::build(session_name.to_owned(), s)
                .same_site(SameSite::Strict)
                .secure(true)
                .http_only(true)
                .finish(),
        )
    } else {
        jar
    };
    Ok(jar)
}

#[instrument(skip_all)]
pub async fn get_handler(
    session: Session,
    Extension(db): Extension<Pool>,
) -> Result<impl IntoResponse, LoginError> {
    if session.verify().await? {
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
pub async fn post_handler(
    Form(cred): Form<LoginCredential>,
    jar: SignedCookieJar,
    mut session: Session,
    Extension(session_name): Extension<SessionCookieName>,
    Extension(db): Extension<Pool>,
) -> Result<(SignedCookieJar, Redirect), LoginError> {
    let conn = db.get().await.unwrap();
    let user_id = cred.validate(&conn).await?;
    if let Some(user_id) = user_id {
        // If successfully logged in but previous session is valid, clear the session
        if session.verify().await? {
            session.purge().await.ok();
        }
        let jar = new_session_to_cookie(&mut session, &session_name.0, jar, user_id).await?;
        Ok((jar, Redirect::to("/")))
    } else {
        Err(LoginError::Unauthorized)
    }
}
