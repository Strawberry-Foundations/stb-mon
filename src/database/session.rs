use crate::database::DATABASE;
use anyhow::Context;
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::iter::repeat_with;
use std::time::UNIX_EPOCH;

pub async fn create_session() -> anyhow::Result<String> {
    let token: String = repeat_with(fastrand::alphanumeric).take(12).collect();
    let mut hasher = Sha256::new();
    hasher.update(&token);
    let hash = hex::encode(hasher.finalize());
    let expiry = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 60 * 60 * 24 * 7; // token is valid for 7 days
    DATABASE
        .get()
        .context("Database not initialized")?
        .lock()
        .await
        .execute(
            "INSERT INTO sessions (token, expiresAt) VALUES (?, ?)",
            params![hash, expiry],
        )?;
    Ok(token)
}
