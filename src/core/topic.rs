use eyre::*;

use deadpool_sqlite::Connection;

use rusqlite::OptionalExtension;
use time::OffsetDateTime;

use crate::error::to_eyre;

use super::{from_row::FromRow, session::Session};

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
    pub async fn query(db: &Connection, session: &Session, id: i64) -> Result<Option<Topic>> {
        let topic = db
            .interact(move |conn| {
                conn.query_row(
                    r#"SELECT * FROM topics WHERE id = ?"#,
                    [id],
                    Topic::try_from_row,
                )
                .optional()
            })
            .await
            .map_err(to_eyre)??;
        Ok(topic.filter(|t| t.is_visible_to(session)))
    }
    pub async fn query_visibility(db: &Connection, session: &Session, id: i64) -> Result<bool> {
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
            .map_err(to_eyre)??
        {
            Ok(Self::topic_is_visible_to(
                author_user_id,
                public,
                &deleted_at,
                session,
            ))
        } else {
            Ok(false)
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
