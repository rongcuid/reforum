use eyre::*;

use deadpool_sqlite::Connection;
use time::OffsetDateTime;

use super::{user::User, session::Session};

pub struct Topic {
    id: i64,
    author: User,
    title: String,
    number_posts: i64,
    public: bool,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    last_updated_by: Option<User>,
    views_from_users: i64,
}

impl Topic {
    /// Queries a topic for a certain role
    pub async fn query(db: &Connection, session: &Session, id: i64) -> Result<Topic> {
        todo!()
    }
}

impl Topic {
    pub async fn posts(&self, db: &Connection, session: &Session) {
        todo!()
    }
}