use argon2::{Argon2, PasswordHash, PasswordVerifier};
use eyre::*;

use deadpool_sqlite::Connection;
use rusqlite::OptionalExtension;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use tracing::instrument;

use crate::{error::to_eyre, telemetry::spawn_blocking_with_tracing};

#[derive(Deserialize, Debug)]
pub struct LoginCredential {
    pub username: String,
    pub password: SecretString,
}

impl LoginCredential {
    pub async fn validate(&self, db: &Connection) -> Result<Option<i64>> {
        let username = self.username.clone();
        let cred = db
            .interact(
                move |conn| -> Result<Option<(i64, String)>, rusqlite::Error> {
                    conn.query_row(
                        r#"SELECT id, phc FROM users WHERE username = ?"#,
                        [username],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
                },
            )
            .await
            .map_err(to_eyre)??;
        if let Some((user_id, phc)) = cred {
            let password = self.password.clone();
            let pass = spawn_blocking_with_tracing(move || {
                verify_password_hash(SecretString::new(phc), password)
            })
            .await??;
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
    let hash = PasswordHash::new(phc.expose_secret())?;
    Ok(Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &hash)
        .is_ok())
}
