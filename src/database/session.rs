use crate::database::DATABASE;
use crate::time_util::current_unix_time;
use anyhow::bail;
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::iter::repeat_with;

pub async fn create() -> anyhow::Result<String> {
    let token: String = repeat_with(fastrand::alphanumeric).take(12).collect();
    let mut hasher = Sha256::new();
    hasher.update(&token);
    let hash = hex::encode(hasher.finalize());
    let expiry = current_unix_time() + 60 * 60 * 24 * 7; // token is valid for 7 days
    DATABASE.lock().await.execute(
        "INSERT INTO sessions (token, expiresAt) VALUES (?, ?)",
        params![hash, expiry],
    )?;

    Ok(token)
}

pub async fn is_valid(token: &str) -> anyhow::Result<bool> {
    if token.len() != 12 {
        return Ok(false);
    }

    let mut hasher = Sha256::new();
    hasher.update(&token);
    let hash = hex::encode(hasher.finalize());
    match DATABASE.lock().await.query_row(
        "SELECT token FROM sessions WHERE (token = ? AND expiresAt > ?)",
        params![hash, current_unix_time()],
        |_| Ok(()),
    ) {
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
        Err(e) => bail!(e),
        _ => {}
    }

    return Ok(true);
}
