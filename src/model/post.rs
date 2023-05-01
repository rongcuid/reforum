use thiserror::*;

use crate::auth::user_role::UserRole;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::model::from_row::FromRow;

pub struct Post {
    pub id: i64,
    pub topic_id: i64,
    pub author_user_id: i64,
    pub author_name: String,
    pub body: String,
    pub post_number: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub last_updated_by: Option<i64>,
}

#[derive(Error, Debug)]
pub enum PostError {
    #[error("post not found")]
    PostNotFound,
    #[error(transparent)]
    DbError(#[from] rusqlite::Error),
}

type Result<T, E = PostError> = std::result::Result<T, E>;

impl Post {
    pub fn query_by_post_id(conn: &Connection, uid: Option<i64>, post_id: i64) -> Result<Post> {
        let post = conn.query_row_and_then(
            r#"SELECT t.* FROM posts p WHERE p.id = ?1"#,
            params![post_id],
            |row| -> Result<Post> {
                let post = Post {
                    id: row.get("id")?,
                    topic_id: row.get("topic_id")?,
                    author_user_id: row.get("author_user_id")?,
                    body: row.get("body")?,
                    post_number: row.get("post_number")?,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                    deleted_at: row.get("deleted_at")?,
                    last_updated_by: row.get("last_updated_by")?,
                };
                Ok(post)
            },
        )?;
        // TODO: deleted post
        Ok(Some(post))
    }
    /// Checks visibility of post, but not the post it belongs to
    fn is_visible_to(&self, role: UserRole) -> bool {
        if self.deleted_at.is_some() {
            // Deleted post is only visible to admin and moderator
            matches!(role, UserRole::Admin | UserRole::Moderator)
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
