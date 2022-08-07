use axum::{extract::Path, response::IntoResponse, *};
use deadpool_sqlite::Pool;
use hyper::StatusCode;

use tracing::instrument;

use crate::core::{session::Session, topic::Topic};

#[instrument(skip_all,fields(id=id))]
pub async fn get_handler(
    Path(id): Path<i64>,
    session: Session,
    Extension(db): Extension<Pool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db = db.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_owned(),
        )
    })?;

    let topic = Topic::query(&db, &session, id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "404 Not Found".to_owned()))?
        .ok_or((StatusCode::FORBIDDEN, "403 Forbidden".to_string()))?;

    Ok(format!("{:?}", topic))
}

#[instrument(skip_all, fields(id=id))]
pub async fn post_handler(
    Path(id): Path<i64>,
    _session: Session,
    Extension(_db): Extension<Pool>,
) -> impl IntoResponse {
    "Not impleneted"
}
