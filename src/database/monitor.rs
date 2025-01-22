use rusqlite::params;

use crate::monitor::Monitor;

use super::DATABASE;

pub async fn add_monitor(service_data: Monitor, delay_mins: u16) -> anyhow::Result<()> {
    tracing::debug!("Adding monitor - service_data: {service_data:?} | delay_mins: {delay_mins}");
    let service_data = rmp_serde::to_vec(&service_data)?;
    DATABASE
        .get()
        .ok_or(anyhow::anyhow!("Database not initialized"))?
        .lock()
        .await
        .execute(
            "INSERT INTO monitors (serviceDataMp, delayMins) VALUES (?, ?)",
            params![service_data, delay_mins],
        )?;

    Ok(())
}

// used for debugging
#[allow(unused)]
pub async fn get_monitor_data_by_id(id: i32) -> Option<Monitor> {
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
