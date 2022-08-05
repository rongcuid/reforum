use std::sync::Arc;

use axum::extract::{FromRequest, RequestParts};
use axum_extra::extract::SignedCookieJar;
use deadpool_sqlite::Connection;
use eyre::*;

use async_trait::async_trait;
use hyper::StatusCode;

use nanoid::nanoid;
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio::sync::Mutex;
use tracing::{error, instrument};

use crate::{error::to_eyre, startup::SessionCookieName};

#[derive(Debug)]
pub struct Session(Option<SessionData>);

impl Session {
    pub fn get(&self) -> Option<&SessionData> {
        self.0.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub user_id: i64,
    pub session_id: String,
}

#[async_trait]
impl<B> FromRequest<B> for Session
where
    B: Send,
{
    type Rejection = StatusCode;

    #[instrument(name = "Session::from_request", skip_all)]
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let jar: SignedCookieJar = req.extract().await.map_err(|_| {
            error!("Signed cookie jar");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let name = req.extensions().get::<SessionCookieName>().ok_or_else(|| {
            error!("Session cookie error");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let session_data = jar
            .get(&name.0)
            .map(|c| serde_json::from_str::<SessionData>(c.value()))
            .map_or(Ok(None), |r| r.map(Some))
            .unwrap_or(None);
        Ok(Self(session_data))
    }
}

#[instrument(skip(db, expires_at))]
pub async fn new_session(
    db: &Connection,
    user_id: i64,
    expires_at: &Option<OffsetDateTime>,
) -> Result<Session> {
    let session_id = nanoid!();
    let hash = Sha256::digest(session_id.as_bytes()).as_slice().to_vec();
    let expires_at = expires_at.clone();
    db.interact(move |conn| -> Result<(), Error> {
        conn.execute(
            r"INSERT INTO user_sessions (id, session_user_id, expires_at) VALUES(?, ?, ?)",
            (hash, user_id, expires_at),
        )?;
        Ok(())
    })
    .await
    .map_err(to_eyre)??;
    Ok(Session(Some(SessionData {
        user_id,
        session_id,
    })))
}

#[instrument(skip_all)]
pub async fn remove_session(db: &Connection, session: &Session) -> Result<()> {
    if let Some(s) = &session.0 {
        let hash = Sha256::digest(s.session_id.as_bytes()).as_slice().to_vec();

        let s = s.clone();
        db.interact(move |conn| -> Result<(), rusqlite::Error> {
            conn.execute(
                r#"DELETE FROM user_sessions WHERE id = ? AND session_user_id = ?"#,
                (hash, s.user_id),
            )?;
            Ok(())
        })
        .await
        .map_err(to_eyre)??;
    }
    Ok(())
}

#[instrument(skip_all)]
pub async fn verify_session(db: &Connection, session: &Session) -> Result<bool> {
    if let Some(s) = &session.0 {
        let hash = Sha256::digest(s.session_id.as_bytes()).as_slice().to_vec();

        let s = s.clone();
        db.interact(move |conn| {
            Ok(conn.query_row(
                r#"SELECT 1 FROM user_sessions WHERE id = ? AND session_user_id = ?"#,
                (hash, s.user_id),
                |_| Ok(true),
            )?)
        })
        .await
        .map_err(to_eyre)?
    } else {
        Ok(false)
    }
}
