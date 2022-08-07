use axum::{http::StatusCode, response::Redirect, Extension};
use axum_extra::extract::SignedCookieJar;
use cookie::Cookie;
use deadpool_sqlite::Pool;
use tracing::instrument;

use crate::{core::session::Session, startup::SessionCookieName};

fn remove_session_from_cookie(session_name: &str, jar: SignedCookieJar) -> SignedCookieJar {
    jar.remove(Cookie::named(session_name.to_owned()))
}

#[instrument(skip_all)]
pub async fn handler(
    jar: SignedCookieJar,
    Extension(pool): Extension<Pool>,
    Extension(session_name): Extension<SessionCookieName>,
    session: Session,
) -> Result<(SignedCookieJar, Redirect), (StatusCode, String)> {
    let jar = remove_session_from_cookie(&session_name.0, jar);
    session.purge().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_owned(),
        )
    })?;

    Ok((jar, Redirect::to("/")))
}
