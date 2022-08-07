use std::sync::Arc;

use eyre::*;

use deadpool_sqlite::Connection;
use sea_query::*;
use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::error::to_eyre;

use super::{from_row::FromRow, session::Session, user::User};

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
            })
            .await
            .map_err(to_eyre)??;
        if topic.is_visible_to(session) {
            Ok(Some(topic))
        } else {
            Ok(None)
        }
    }
    fn is_visible_to(&self, session: &Session) -> bool {
        if self.deleted_at.is_some() {
            // Deleted topic is only visible to admin and moderator
            session.is_admin() || session.is_moderator()
        } else if !self.public {
            // Hidden post is only visible to admin, moderator, and topic author
            session.is_admin()
                || session.is_moderator()
                || session.user_id() == Some(self.author_user_id)
        } else {
            // Public post is visible to everyone
            true
        }
    }
}

impl Topic {
    pub async fn author(&self, db: &Connection, session: &Session) {
        todo!()
    }
    pub async fn last_updated_by(&self, db: &Connection, session: &Session) {
        todo!()
    }
    pub async fn posts(&self, db: &Connection, session: &Session) {
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
