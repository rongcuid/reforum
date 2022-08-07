use axum::{extract::Path, response::IntoResponse, *};
use deadpool_sqlite::Pool;
use tracing::instrument;

use crate::core::session::Session;

#[instrument(skip_all,fields(id=id))]
pub async fn get_handler(
    Path(id): Path<i64>,
    session: Session,
    Extension(db): Extension<Pool>,
) -> impl IntoResponse {
    "Not implemented"
}

#[instrument(skip_all, fields(id=id))]
pub async fn post_handler(
    Path(id): Path<i64>,
    session: Session,
    Extension(db): Extension<Pool>,
) -> impl IntoResponse {
    "Not impleneted"
}
