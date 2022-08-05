use axum::{http::StatusCode, response::IntoResponse, Extension};
use axum_extra::extract::SignedCookieJar;
use cookie::{Cookie, SameSite};
use deadpool_sqlite::{Pool, Status};
use secrecy::SecretString;
use time::OffsetDateTime;
use tracing::instrument;

use nanoid::nanoid;

use crate::{core::session::new_session, startup::SessionCookieName};

// #[instrument(skip_all)]
pub async fn handler(
    jar: SignedCookieJar,
    Extension(session_name): Extension<SessionCookieName>,
    Extension(db): Extension<Pool>,
) -> Result<(SignedCookieJar, String), (StatusCode, String)> {
    let id = nanoid!();
    let session = new_session(
        &db.get().await.unwrap(),
        1,
        &Some(OffsetDateTime::now_utc()),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let jar = if let Some(s) = serde_json::to_string(&session.get()).ok() {
        jar.add(
            Cookie::build(session_name.0.clone(), s)
                .same_site(SameSite::Strict)
                .secure(true)
                .http_only(true)
                .finish(),
        )
    } else {
        jar
    };
    Ok((
        jar,
        format!("Stub implementation, inserting random session ID: {}", id),
    ))
}
