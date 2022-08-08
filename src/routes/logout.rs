use axum::{http::StatusCode, response::Redirect, Extension};
use axum_extra::extract::SignedCookieJar;
use cookie::Cookie;
use tracing::instrument;

use crate::{core::session::{Session, SessionError}, startup::SessionCookieName};

fn remove_session_from_cookie(session_name: &str, jar: SignedCookieJar) -> SignedCookieJar {
    jar.remove(Cookie::named(session_name.to_owned()))
}

#[instrument(skip_all)]
pub async fn handler(
    jar: SignedCookieJar,
    Extension(session_name): Extension<SessionCookieName>,
    session: Session,
) -> Result<(SignedCookieJar, Redirect), SessionError> {
    let jar = remove_session_from_cookie(&session_name.0, jar);
    session.purge().await?;

    Ok((jar, Redirect::to("/")))
}
