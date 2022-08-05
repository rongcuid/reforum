use axum::{http::StatusCode, response::IntoResponse, Extension};
use secrecy::SecretString;
use tracing::instrument;

use nanoid::nanoid;

use crate::core::session::insert_session;

#[instrument]
pub async fn handler(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
) -> Result<String, (StatusCode, String)> {
    let id = nanoid!();
    insert_session(
        &db,
        1,
        &SecretString::new(id.clone()),
        &Some(chrono::offset::Utc::now()),
    )
    .await
    .or(Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Insert session error".to_owned(),
    )))?;
    Ok(format!(
        "Stub implementation, inserting random session ID: {}",
        id
    ))
}
