use axum::response::{IntoResponse, Response};
use axum_sessions::extractors::ReadableSession;
use chrono::{DateTime, Utc};
use hyper::StatusCode;
// use eyre::*;
use thiserror::*;

use rusqlite::{params, Connection, OptionalExtension};

use super::{from_row::FromRow, post::Post};

#[derive(Error, Debug)]
pub enum TopicError {
    #[error("topic `{0}` not found")]
    NotFound(i64),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error(transparent)]
    Other(#[from] std::io::Error),
}

impl IntoResponse for TopicError {
    fn into_response(self) -> Response {
        match self {
            TopicError::NotFound(_) => (StatusCode::NOT_FOUND, "404 not found"),
            TopicError::Forbidden(_) => (StatusCode::FORBIDDEN, "403 forbidden"),
            TopicError::DeadpoolError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
            TopicError::RusqliteError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
            TopicError::InternalError(_) => (
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

type Result<T, E = TopicError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Topic {
    pub id: i64,
    pub author_user_id: i64,
    pub title: String,
    pub number_posts: i64,
    pub public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub last_updated_by: Option<i64>,
    pub views_from_users: i64,
}

impl Topic {
    /// Queries a topic for a certain role
    pub async fn query(
        conn: &Connection,
        session: &ReadableSession,
        id: i64,
    ) -> Result<Topic, TopicError> {
        let topic = conn
            .query_row(
                r#"SELECT * FROM topics WHERE id = ?"#,
                [id],
                Topic::try_from_row,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => TopicError::NotFound(id),
                e => e.into(),
            })?;
        if topic.is_visible_to(session) {
            Ok(topic)
        } else {
            Err(TopicError::Forbidden(format!(
                "{} cannot view topic {}",
                id,
                session.cred_str()
            )))
        }
    }
    pub async fn query_visibility(
        db: &Connection,
        session: &ReadableSession,
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
    fn is_visible_to(&self, session: &ReadableSession) -> bool {
        Self::topic_is_visible_to(self.author_user_id, self.public, &self.deleted_at, session)
    }
    fn topic_is_visible_to(
        author_user_id: i64,
        public: bool,
        deleted_at: &Option<DateTime<Utc>>,
        session: &ReadableSession,
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
    pub async fn insert_topic(
        db: &Connection,
        session: &ReadableSession,
        title: &str,
        public: bool,
        body: bool,
    ) -> Result<(Self, Post), TopicError> {
        if !session.can_post() {
            return Err(TopicError::Forbidden(format!(
                "`{}` cannot post topic",
                session.cred_str()
            )));
        }
        let user_id = session.user_id().ok_or(TopicError::InternalError(
            "Session User ID failed".to_owned(),
        ))?;
        let title = title.to_owned();
        let body = body.to_owned();
        let (topic, post) = db
            .interact(move |conn| -> Result<(Topic, Post), rusqlite::Error> {
                let tx = conn.transaction()?;
                let topic = tx.query_row(
                    r#"
                INSERT INTO topics(author_user_id, title, public)
                VALUES (?, ?, ?)
                RETURNING *
                "#,
                    params![user_id, title, public],
                    Topic::try_from_row,
                )?;
                let post = tx.query_row(
                    r#"
                    INSERT INTO posts(topic_id, author_user_id, body, public)
                    VALUES (?, ?, ?, ?)
                    RETURNING *
                    "#,
                    params![topic.id, user_id, body, public],
                    Post::try_from_row,
                )?;
                tx.commit()?;
                Ok((topic, post))
            })
            .await
            .map_err(|_| TopicError::DeadpoolError)??;
        Ok((topic, post))
    }
}

impl Topic {
    pub async fn author(&self, _db: &Connection, _session: &ReadableSession) {
        todo!()
    }
    pub async fn last_updated_by(&self, _db: &Connection, _session: &ReadableSession) {
        todo!()
    }
    pub async fn posts(&self, _db: &Connection, _session: &ReadableSession) {
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
