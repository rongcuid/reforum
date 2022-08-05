use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Extension,
};
use axum_extra::extract::SignedCookieJar;
use cookie::{Cookie, SameSite};
use deadpool_sqlite::{Pool, Status};
use secrecy::SecretString;
use time::OffsetDateTime;
use tracing::instrument;

use nanoid::nanoid;

use crate::{core::session::{new_session, Session}, startup::SessionCookieName};

async fn new_session_to_cookie(
    db: &Pool,
    session_name: &str,
    jar: SignedCookieJar,
) -> Result<SignedCookieJar, (StatusCode, String)> {
    let session = new_session(
        &db.get().await.unwrap(),
        1,
        &Some(OffsetDateTime::now_utc()),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let jar = if let Some(s) = serde_json::to_string(&session.get()).ok() {
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
pub async fn handler(
    jar: SignedCookieJar,
    session: Session,
    Extension(session_name): Extension<SessionCookieName>,
    Extension(db): Extension<Pool>,
) -> impl IntoResponse {
    if session.get().is_some() {
        return Err((StatusCode::FORBIDDEN, "Already logged in".to_owned()))
    }
    let jar = new_session_to_cookie(&db, &session_name.0, jar).await?;
    Ok((jar, Redirect::to("/")))
}
