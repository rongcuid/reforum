use crate::auth::user_role::{AuthorizationError, UserRole};
use crate::configuration::SQLite3Settings;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, RequestPartsExt};
use axum_sessions::extractors::ReadableSession;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserAuthError {
    #[error("Not logged in")]
    NotLoggedIn,
    #[error("Internal error")]
    InternalError,
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
}

impl From<AuthorizationError> for UserAuthError {
    fn from(value: AuthorizationError) -> Self {
        match value {
            AuthorizationError::RusqliteError(e) => e.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserAuth {
    pub id: i64,
    pub role: UserRole,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserAuth {
    type Rejection = UserAuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = parts
            .extract::<Extension<SQLite3Settings>>()
            .await
            .map_err(|_| UserAuthError::InternalError)?;
        let session = parts
            .extract::<Option<ReadableSession>>()
            .await
            .map_err(|_| UserAuthError::InternalError)?;
        if session.is_none() {
            return Err(UserAuthError::NotLoggedIn);
        }
        let session = session.unwrap();
        let uid = session
            .get::<i64>("uid")
            .ok_or(UserAuthError::InternalError)?;

        let conn = db.connect()?;
        let role = UserRole::from_db(&conn, uid)?;
        Ok(Self { id: uid, role })
    }
}

impl IntoResponse for UserAuthError {
    fn into_response(self) -> Response {
        match self {
            UserAuthError::RusqliteError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
            UserAuthError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
            UserAuthError::NotLoggedIn => (StatusCode::FORBIDDEN, "403 Forbidden"),
        }
        .into_response()
    }
}
