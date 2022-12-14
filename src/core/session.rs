use thiserror::*;

use axum::{
    extract::{FromRequest, RequestParts},
    response::IntoResponse,
    Extension,
};
use axum_extra::extract::SignedCookieJar;
use deadpool_sqlite::{Connection, Pool};

use async_trait::async_trait;
use hyper::StatusCode;

use nanoid::nanoid;

use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use tracing::{error, instrument};

use crate::{startup::SessionCookieName};

use super::authorization::UserRole;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    InteractError(#[from] deadpool_sqlite::InteractError),
    #[error(transparent)]
    PoolError(#[from] deadpool_sqlite::PoolError),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
    #[error(transparent)]
    Other(#[from] eyre::Error),
}

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionError::RusqliteError(rusqlite::Error::QueryReturnedNoRows) => {
                (StatusCode::NOT_FOUND, "404 Not Found".to_owned())
            }
            e => {
                error!("{}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "500 Internal Server Error".to_owned(),
                )
            }
        }
        .into_response()
    }
}

#[derive(Debug)]
pub struct Session {
    /// Current authenticated session data
    data: Option<SessionData>,
    pool: Pool,
}

impl Session {
    /// Get current session
    pub fn get(&self) -> Option<&SessionData> {
        self.data.as_ref()
    }
    /// Create and use a new session
    pub async fn insert(
        &mut self,
        user_id: i64,
        expires_at: &Option<OffsetDateTime>,
    ) -> Result<SessionData, SessionError> {
        let conn = self.pool.get().await?;
        self.purge().await?;
        let session = new_session(&conn, user_id, expires_at).await?;
        self.data = Some(session.clone());
        Ok(session)
    }
    pub async fn purge(&self) -> Result<(), SessionError> {
        let conn = self.pool.get().await?;
        remove_session(&conn, &self.data).await
    }
    pub async fn verify(&self) -> Result<bool, SessionError> {
        if let Some(s) = self.data.as_ref() {
            let conn = self.pool.get().await?;
            verify_session(&conn, s).await
        } else {
            Ok(false)
        }
    }
    pub async fn renew(&self) {}

    pub fn user_id(&self) -> Option<i64> {
        self.data.as_ref().map(|x| x.user_id)
    }

    pub fn is_anonymous(&self) -> bool {
        self.data.is_none()
    }

    pub fn is_viewer(&self) -> bool {
        self.get()
            .map(|x| matches!(x.role, UserRole::Viewer))
            .unwrap_or(false)
    }

    pub fn is_author(&self) -> bool {
        self.get()
            .map(|x| matches!(x.role, UserRole::Author))
            .unwrap_or(false)
    }

    pub fn is_moderator(&self) -> bool {
        self.get()
            .map(|x| matches!(x.role, UserRole::Moderator))
            .unwrap_or(false)
    }

    pub fn is_admin(&self) -> bool {
        self.get()
            .map(|x| matches!(x.role, UserRole::Admin))
            .unwrap_or(false)
    }

    pub fn can_post(&self) -> bool {
        self.is_author() || self.is_moderator() || self.is_admin()
    }

    pub fn cred_str(&self) -> String {
        if let Some(d) = self.get() {
            format!("{}", d.user_id)
        } else {
            "Anonymous".to_owned()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub user_id: i64,
    pub session_id: String,
    pub role: UserRole,
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
        let Extension(pool): Extension<Pool> = req.extract().await.map_err(|_| {
            error!("DB Pool");
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
        Ok(Self {
            data: session_data,
            pool,
        })
    }
}

#[instrument(skip(db, expires_at))]
async fn new_session(
    db: &Connection,
    user_id: i64,
    expires_at: &Option<OffsetDateTime>,
) -> Result<SessionData, SessionError> {
    let session_id = nanoid!();
    let hash = Sha256::digest(session_id.as_bytes()).as_slice().to_vec();
    let expires_at = *expires_at;
    db.interact(move |conn| -> Result<(), rusqlite::Error> {
        conn.execute(
            r"INSERT INTO user_sessions (id, session_user_id, expires_at) VALUES(?, ?, ?)",
            (hash, user_id, expires_at),
        )?;
        Ok(())
    })
    .await??;
    let role = UserRole::from_db(db, user_id).await?;
    Ok(SessionData {
        user_id,
        session_id,
        role,
    })
}

#[instrument(skip_all)]
async fn remove_session(
    db: &Connection,
    session: &Option<SessionData>,
) -> Result<(), SessionError> {
    if let Some(s) = &session {
        let hash = Sha256::digest(s.session_id.as_bytes()).as_slice().to_vec();

        let s = s.clone();
        db.interact(move |conn| -> Result<(), rusqlite::Error> {
            conn.execute(
                r#"DELETE FROM user_sessions WHERE id = ? AND session_user_id = ?"#,
                (hash, s.user_id),
            )?;
            Ok(())
        })
        .await??;
    }
    Ok(())
}

#[instrument(skip_all)]
async fn verify_session(db: &Connection, session: &SessionData) -> Result<bool, SessionError> {
    let hash = Sha256::digest(session.session_id.as_bytes())
        .as_slice()
        .to_vec();

    let s = session.clone();
    db.interact(move |conn| {
        Ok(conn
            .query_row(
                r#"SELECT 1 FROM user_sessions WHERE id = ? AND session_user_id = ?"#,
                (hash, s.user_id),
                |_| Ok(true),
            )
            .optional()?
            .is_some())
    })
    .await?
}
