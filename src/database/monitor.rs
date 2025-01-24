use anyhow::bail;
use rusqlite::params;

use crate::monitor::Monitor;

use super::DATABASE;

pub async fn add(service_data: Monitor, interval_mins: u16) -> anyhow::Result<()> {
    tracing::debug!(
        "Adding monitor - service_data: {service_data:?} | interval_mins: {interval_mins}"
    );
    let service_data = rmp_serde::to_vec(&service_data)?;
    DATABASE
        .get()
        .ok_or(anyhow::anyhow!("Failed to get database"))?
        .lock()
        .await
        .execute(
            "INSERT INTO monitors (serviceDataMp, intervalMins) VALUES (?, ?)",
            params![service_data, interval_mins],
        )?;

    Ok(())
}

// used for debugging
#[allow(unused)]
pub async fn get_by_id(id: i32) -> Option<Monitor> {
    let lock = DATABASE.get()?.lock().await;
    let bytes: Vec<u8> = lock
        .query_row(
            "SELECT serviceDataMp FROM monitors WHERE id = ?",
            [id],
            |r| r.get(0),
        )
        .ok()?;
    Some(rmp_serde::from_slice(&bytes).unwrap())
}
