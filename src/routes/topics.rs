use axum::{extract::{Path, Query}, response::IntoResponse, *};
use deadpool_sqlite::Pool;
use hyper::StatusCode;

use tracing::instrument;

use crate::core::{
    session::Session,
    topic::{Topic, TopicError}, filter::Pagination,
};

/// Get a topic
#[instrument(skip_all,fields(id=id))]
pub async fn get_handler(
    Path(id): Path<i64>,
    Query(pagination): Query<Pagination>,
    session: Session,
    Extension(db): Extension<Pool>,
) -> Result<impl IntoResponse, TopicError> {
    let db = db.get().await.map_err(|_| TopicError::DeadpoolError)?;

    let topic = Topic::query(&db, &session, id).await?;
    Ok(format!("Got topic {}: {}",topic.id,topic.title))
}

/// Post a new reply to a topic
#[instrument(skip_all, fields(id=id))]
pub async fn post_handler(
    Path(id): Path<i64>,
    _session: Session,
    Extension(_db): Extension<Pool>,
) -> impl IntoResponse {
    "Not impleneted"
}
