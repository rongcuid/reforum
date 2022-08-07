use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Extension, Form,
};
use axum_extra::extract::SignedCookieJar;
use cookie::{Cookie, SameSite};
use deadpool_sqlite::{Connection, Pool};
use maud::html;

use time::{Duration, OffsetDateTime};
use tracing::instrument;

use crate::{
    core::{
        authentication::LoginCredential,
        session::{new_session, remove_session, verify_session, Session},
    },
    startup::SessionCookieName,
};

async fn new_session_to_cookie(
    conn: &Connection,
    session_name: &str,
    jar: SignedCookieJar,
    user_id: i64,
) -> Result<SignedCookieJar, (StatusCode, String)> {
    let expires_at = Some(OffsetDateTime::now_utc() + Duration::weeks(4));
    let session = new_session(conn, user_id, &expires_at)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
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
pub async fn get_handler(session: Session, Extension(db): Extension<Pool>) -> impl IntoResponse {
    if verify_session(&db.get().await.unwrap(), &session)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_owned(),
            )
        })?
    {
        return Err((StatusCode::FORBIDDEN, "Already logged in".to_owned()));
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
    session: Session,
    Extension(session_name): Extension<SessionCookieName>,
    Extension(db): Extension<Pool>,
) -> Result<(SignedCookieJar, Redirect), (StatusCode, String)> {
    let conn = db.get().await.unwrap();
    let user_id = cred.validate(&conn).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_owned(),
        )
    })?;
    if let Some(user_id) = user_id {
        // If successfully logged in but previous session is valid, clear the session
        if verify_session(&conn, &session).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_owned(),
            )
        })? {
            remove_session(&conn, &session).await.ok();
        }
        let jar = new_session_to_cookie(&conn, &session_name.0, jar, user_id).await?;
        Ok((jar, Redirect::to("/")))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "Incorrect username or password".to_owned(),
        ))
    }
}
