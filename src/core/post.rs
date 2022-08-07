use eyre::*;

use deadpool_sqlite::Connection;
use time::OffsetDateTime;

use super::{session::Session, user::User};

pub struct Post {
    id: i64,
    topic_id: i64,
    author: User,
    body: String,
    post_number: i64,
    public: bool,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    last_updated_by: Option<User>,
}

impl Post {
    /// Queries a topic for a certain role
    pub async fn query(_db: &Connection, _session: &Session, _id: i64) -> Result<Post> {
        todo!()
    }
}

impl Post {}
