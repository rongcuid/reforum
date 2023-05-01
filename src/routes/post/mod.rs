use chrono::{DateTime, Utc};
use maud::html;
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::session::Session;
use poem::web::{Data, Html, Path};
use poem::*;
use rusqlite::{params, Connection};
use serde::de::StdError;
use std::error::Error;
use thiserror::Error;
use tracing::instrument;

use crate::configuration::SQLite3Settings;

#[derive(Error, Debug)]
pub enum PostError {
    #[error("post deleted")]
    Deleted,
    #[error(transparent)]
    DbError(#[from] rusqlite::Error),
}

impl ResponseError for PostError {
    fn status(&self) -> StatusCode {
        match self {
            PostError::Deleted => StatusCode::GONE,
            PostError::DbError(rusqlite::Error::QueryReturnedNoRows) => StatusCode::NOT_FOUND,
            PostError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn as_response(&self) -> Response
    where
        Self: StdError + Send + Sync + 'static,
    {
        let body = match self {
            PostError::Deleted => html! { p { "Post deleted" } },
            PostError::DbError(rusqlite::Error::QueryReturnedNoRows) => {
                html! { p { "Post not found" }}
            }
            PostError::DbError(_) => html! { p {"Internal server error"} },
        }
        .0;
        Response::builder().status(self.status()).body(body)
    }
}

/// Get a post
#[instrument(skip_all)]
#[handler]
pub async fn get_handler(
    Path(post_id): Path<i64>,
    db: Data<&SQLite3Settings>,
    session: &Session,
) -> Result<impl IntoResponse, PostError> {
    let conn = db.connect()?;
    let uid = session.get::<i64>("uid");

    conn.query_row_and_then(
        r#"
    SELECT 
        p.*, u.username, u.is_moderator
    FROM posts p JOIN users u ON p.author_user_id = u.id
    WHERE p.id = ?1
    "#,
        params![post_id],
        |r| {
            let deleted = r.get::<_, Option<DateTime<Utc>>>("deleted_at")?.is_none();
            let username = r.get::<_, String>("username")?;
            let is_admin = username == "admin";
            let is_mod = r.get::<_, bool>("is_moderator")?;
            if deleted && !is_admin && !is_mod {
                Err(PostError::Deleted)
            } else {
                let body: String = r.get("body")?;
                Ok(Html(
                    html! {
                        p { (body) }
                    }
                    .0,
                ))
            }
        },
    )
}
