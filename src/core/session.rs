use std::sync::Arc;

use axum::extract::{FromRequest, RequestParts};
use deadpool_sqlite::Connection;
use eyre::*;

use async_trait::async_trait;
use hyper::StatusCode;

use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, PrimitiveDateTime};
use tracing::{error, instrument};

#[derive(Debug)]
pub struct Session(Option<SessionData>);

impl Session {
    pub fn get(&self) -> Option<&SessionData> {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct SessionData {
    pub user_id: i64,
}

#[async_trait]
impl<B> FromRequest<B> for Session
where
    B: Send,
{
    type Rejection = StatusCode;

    #[instrument(name = "session_extractor", skip_all)]
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(Self(None))
    }
}

#[instrument(skip(db, expires_at))]
pub async fn insert_session(
    db: &Connection,
    user_id: i64,
    session_id: &Secret<String>,
    expires_at: &Option<OffsetDateTime>,
) -> Result<()> {
    let hash = Sha256::digest(session_id.expose_secret().as_bytes())
        .as_slice()
        .to_vec();
    let expires_at = expires_at.clone();
    db.interact(move |conn| -> Result<(), Error> {
        conn.execute(
            r"INSERT INTO user_sessions (id, session_user_id, expires_at) VALUES(?, ?, ?)",
            (hash, user_id, expires_at),
        )?;
        Ok(())
    })
    .await.unwrap()
    .map_err(|e| {
        error!("{}", e);
        eyre!(e.to_string())
    })?;
    Ok(())
}
