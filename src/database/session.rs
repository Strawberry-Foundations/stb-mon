use crate::database::DATABASE;
use anyhow::{Context, bail};
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
        .context("Failed to get database")?
        .lock()
        .await
        .execute(
            "INSERT INTO sessions (token, expiresAt) VALUES (?, ?)",
            params![hash, expiry],
        )?;

    Ok(token)
}

pub async fn is_valid_session(token: &str) -> anyhow::Result<bool> {
    if token.len() != 12 {
        return Ok(false);
    }

    let mut hasher = Sha256::new();
    hasher.update(&token);
    let hash = hex::encode(hasher.finalize());
    let time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    match DATABASE
        .get()
        .context("Failed to get database")?
        .lock()
        .await
        .query_row(
            "SELECT token FROM sessions WHERE (token = ? AND expiresAt > ?)",
            params![hash, time],
            |_| Ok(()),
        ) {
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
        Err(e) => bail!(e),
        _ => {}
    }

    return Ok(true);
}
