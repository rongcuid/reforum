use argon2::{Argon2, PasswordHash, PasswordVerifier};
use color_eyre::eyre::WrapErr;

use crate::configuration::SQLite3Settings;
use rusqlite::OptionalExtension;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use thiserror::Error;
use tracing::instrument;

use crate::telemetry::spawn_blocking_with_tracing;

#[derive(Deserialize, Debug)]
pub struct LoginCredential {
    pub username: String,
    pub password: SecretString,
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Internal error")]
    InternalError,
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
}

type Result<T, E = LoginError> = std::result::Result<T, E>;

impl LoginCredential {
    pub async fn validate(&self, db: &SQLite3Settings) -> Result<Option<i64>> {
        let username = self.username.clone();
        let conn = db.connect()?;
        let cred = conn
            .query_row(
                r#"SELECT id, phc FROM users WHERE username = ?"#,
                [username],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((user_id, phc)) = cred {
            let password = self.password.clone();
            let pass = spawn_blocking_with_tracing(move || {
                verify_password_hash(SecretString::new(phc), password)
            })
            .await
            .map_err(|_| LoginError::InternalError)??;
            if pass {
                Ok(Some(user_id))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

#[instrument(skip_all)]
fn verify_password_hash(phc: SecretString, password: SecretString) -> Result<bool> {
    let hash = PasswordHash::new(phc.expose_secret()).map_err(|_| LoginError::InternalError)?;
    Ok(Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &hash)
        .is_ok())
}
