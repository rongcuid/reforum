use poem::http::{header, StatusCode};
use poem::session::Session;
use poem::*;

use tracing::instrument;

#[instrument(skip_all)]
#[handler]
pub async fn handler(session: &Session) -> impl IntoResponse {
    session.purge();
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/")
        .finish()
}
