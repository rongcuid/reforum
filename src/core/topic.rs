use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
// use eyre::*;
use thiserror::*;

use deadpool_sqlite::Connection;

use rusqlite::OptionalExtension;
use time::OffsetDateTime;

use crate::error::to_eyre;

use super::{from_row::FromRow, session::Session};

#[derive(Error, Debug)]
pub enum TopicError {
    #[error("topic `{0}` not found")]
    NotFound(i64),
    #[error("topic `{0}` forbidden for {1}")]
    Forbidden(i64, String),
    #[error("deadpool error")]
    DeadpoolError,
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
    #[error(transparent)]
    Other(#[from] std::io::Error),
}

impl IntoResponse for TopicError {
    fn into_response(self) -> Response {
        match self {
            TopicError::NotFound(_) => (StatusCode::NOT_FOUND, "404 not found"),
            TopicError::Forbidden(_, _) => (StatusCode::FORBIDDEN, "403 forbidden"),
            TopicError::DeadpoolError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
            TopicError::RusqliteError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
            TopicError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
        }
        .into_response()
    }
}

#[derive(Debug)]
pub struct Topic {
    pub id: i64,
    pub author_user_id: i64,
    pub title: String,
    pub number_posts: i64,
    pub public: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
    pub last_updated_by: Option<i64>,
    pub views_from_users: i64,
}

impl Topic {
    /// Queries a topic for a certain role
    pub async fn query(db: &Connection, session: &Session, id: i64) -> Result<Topic, TopicError> {
        let topic = db
            .interact(move |conn| {
                conn.query_row(
                    r#"SELECT * FROM topics WHERE id = ?"#,
                    [id],
                    Topic::try_from_row,
                )
            })
            .await
            .map_err(|_| TopicError::DeadpoolError)?
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => TopicError::NotFound(id),
                e => e.into(),
            })?;
        if topic.is_visible_to(&session) {
            Ok(topic)
        } else {
            Err(TopicError::Forbidden(id, session.cred_str()))
        }
    }
    pub async fn query_visibility(
        db: &Connection,
        session: &Session,
        id: i64,
    ) -> Result<bool, TopicError> {
        if let Some((author_user_id, public, deleted_at)) = db
            .interact(move |conn| {
                conn.query_row(
                    r#"SELECT author_user_id, public, deleted_at FROM topics WHERE id = ?"#,
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
            })
            .await
            .map_err(|_| TopicError::DeadpoolError)??
        {
            Ok(Self::topic_is_visible_to(
                author_user_id,
                public,
                &deleted_at,
                session,
            ))
        } else {
            Err(TopicError::NotFound(id))
        }
    }
    fn is_visible_to(&self, session: &Session) -> bool {
        Self::topic_is_visible_to(self.author_user_id, self.public, &self.deleted_at, session)
    }
    fn topic_is_visible_to(
        author_user_id: i64,
        public: bool,
        deleted_at: &Option<OffsetDateTime>,
        session: &Session,
    ) -> bool {
        if deleted_at.is_some() {
            // Deleted topic is only visible to admin and moderator
            session.is_admin() || session.is_moderator()
        } else if !public {
            // Hidden post is only visible to admin, moderator, and topic author
            session.is_admin()
                || session.is_moderator()
                || session.user_id() == Some(author_user_id)
        } else {
            // Public post is visible to everyone
            true
        }
    }
}

impl Topic {
    pub async fn author(&self, _db: &Connection, _session: &Session) {
        todo!()
    }
    pub async fn last_updated_by(&self, _db: &Connection, _session: &Session) {
        todo!()
    }
    pub async fn posts(&self, _db: &Connection, _session: &Session) {
        todo!()
    }
}

impl FromRow for Topic {
    fn try_from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            author_user_id: row.get("author_user_id")?,
            title: row.get("title")?,
            number_posts: row.get("number_posts")?,
            public: row.get("public")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            deleted_at: row.get("deleted_at")?,
            last_updated_by: row.get("last_updated_by")?,
            views_from_users: row.get("views_from_users")?,
        })
    }
}
