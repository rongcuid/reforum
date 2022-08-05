use axum::{http::StatusCode, response::IntoResponse, Extension};
use deadpool_sqlite::{Pool, Status};
use secrecy::SecretString;
use time::OffsetDateTime;
use tracing::instrument;

use nanoid::nanoid;

use crate::core::session::insert_session;

#[instrument(skip_all)]
pub async fn handler(Extension(db): Extension<Pool>) -> Result<String, (StatusCode, String)> {
    let id = nanoid!();
    insert_session(
        &db.get().await.unwrap(),
        1,
        &SecretString::new(id.clone()),
        &Some(OffsetDateTime::now_utc()),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(format!(
        "Stub implementation, inserting random session ID: {}",
        id
    ))
}
