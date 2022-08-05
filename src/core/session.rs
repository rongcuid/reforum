use axum::extract::{FromRequest, RequestParts};
use eyre::*;

use async_trait::async_trait;
use hyper::StatusCode;
use tracing::instrument;

#[derive(Debug)]
pub struct Session(Option<SessionData>);

impl Session {
    pub fn get(&self) -> Option<&SessionData> {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct SessionData {
    pub user_id: i64,
}

#[async_trait]
impl<B> FromRequest<B> for Session
where
    B: Send,
{
    type Rejection = StatusCode;

    #[instrument(name = "session_extractor", skip_all)]
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(Self(None))
    }
}
