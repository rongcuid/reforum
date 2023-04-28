use crate::configuration::SQLite3Settings;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthorizationError {
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
}

type Result<T, E = AuthorizationError> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum UserRole {
    /// A banned user may have partial viewing permission
    Banned,
    /// A regular user with read-only permission
    Viewer,
    /// A user who can post topics, posts, and replies
    Author,
    /// A user who can edit/delete others' content
    Moderator,
    /// Superuser
    Admin,
}

impl UserRole {
    pub fn from_db(conn: &Connection, user_id: i64) -> Result<Self> {
        if user_id == 1 {
            return Ok(Self::Admin);
        }
        Ok(conn.query_row(
            include_str!("sql/user_moderation_status_by_id.sql"),
            [user_id],
            Self::from_row,
        )?)
    }
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let banned_at: Option<DateTime<Utc>> = row.get("banned_at")?;
        let muted_until: Option<DateTime<Utc>> = row.get("muted_until")?;
        let moderator_assigned_at: Option<DateTime<Utc>> = row.get("moderator_assigned_at")?;
        let now = Utc::now();
        if banned_at.map(|b| b < now) == Some(true) {
            Ok(Self::Banned)
        } else if muted_until.map(|m| now < m) == Some(true) {
            Ok(Self::Viewer)
        } else if moderator_assigned_at.map(|m| m < now) == Some(true) {
            Ok(Self::Moderator)
        } else {
            Ok(Self::Author)
        }
    }
}
