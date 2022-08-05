use axum::extract::{FromRequest, RequestParts};
use entity::sessions;
use eyre::*;

use async_trait::async_trait;
use hyper::StatusCode;
use migration::OnConflict;
use sea_orm::{prelude::*, ActiveValue};

use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
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

#[instrument]
pub async fn insert_session(
    db: &DatabaseConnection,
    user_id: i64,
    session_id: &Secret<String>,
    expires_at: &Option<DateTimeUtc>,
) -> Result<(), DbErr> {
    let hash = Sha256::digest(session_id.expose_secret().as_bytes())
        .as_slice()
        .to_vec();
    let new = sessions::ActiveModel {
        id: ActiveValue::Set(hash),
        user_id: ActiveValue::Set(user_id),
        expires_at: ActiveValue::Set(*expires_at),
    };
    // TODO: on conflict rollback
    sessions::Entity::insert(new)
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(db)
        .await?;
    Ok(())
}
