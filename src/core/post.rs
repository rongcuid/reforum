use eyre::*;

use deadpool_sqlite::Connection;
use time::OffsetDateTime;

use crate::error::to_eyre;

use super::{from_row::FromRow, session::Session, topic::Topic, user::User};

pub struct Post {
    pub id: i64,
    pub topic_id: i64,
    pub author_user_id: i64,
    pub body: String,
    pub post_number: i64,
    pub public: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
    pub last_updated_by: Option<i64>,
}

impl Post {
    pub async fn query_by_topic_id(db: &Connection, session: &Session, topic_id: i64) -> Result<Vec<Post>> {
        todo!()
    }
    /// Checks visibility of post, but not the topic it belongs to
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

impl FromRow for Post {
    fn try_from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            topic_id: row.get("topic_id")?,
            author_user_id: row.get("author_user_id")?,
            post_number: row.get("post_number")?,
            public: row.get("public")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            deleted_at: row.get("deleted_at")?,
            last_updated_by: row.get("last_updated_by")?,
            body: row.get("body")?,
        })
    }
}
