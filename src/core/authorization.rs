use deadpool_sqlite::Connection;
use eyre::*;

use rusqlite::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::error::to_eyre;

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
    pub async fn from_db(conn: &Connection, user_id: i64) -> Result<Self> {
        if user_id == 1 {
            return Ok(Self::Admin);
        }
        let role = conn.interact(move |conn| {
            conn.query_row(
                include_str!("sql/user_moderation_status_by_id.sql"),
                [user_id],
                Self::from_row
            )
        })
        .await
        .map_err(to_eyre)??;
        Ok(role)
    }
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let banned_at: Option<OffsetDateTime> = row.get("banned_at")?;
        let muted_until: Option<OffsetDateTime> = row.get("muted_until")?;
        let moderator_assigned_at: Option<OffsetDateTime> = row.get("moderator_assigned_at")?;
        let now = OffsetDateTime::now_utc();
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
