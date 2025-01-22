use rusqlite::params;
use std::time::UNIX_EPOCH;

use super::DATABASE;

struct MonitorRecord {
    time_checked: i32,
    result: RecordResult,
    // None = timeout/error
    response_time: Option<i32>,
    pub monitor_id: i32,
    // Info about the result, depends on service and result type
    pub info: String,
}

#[derive(Debug)]
#[repr(u8)]
pub enum RecordResult {
    Ok,
    Unexpected,
    Down,
    Err,
}

pub async fn add_record(
    result: RecordResult,
    response_time: Option<u64>,
    monitor_id: i32,
    info: String,
) -> anyhow::Result<()> {
    tracing::debug!("Adding record - result: {result:?} | response_time: {response_time:?} | monitor_id: {monitor_id} | info: {info}");
    let time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    DATABASE
        .get()
        .ok_or(anyhow::anyhow!("Database not initialized"))?
        .lock()
        .await
        .execute(
            "INSERT INTO records (monitorId, result, responseDeltaMs, checkedAt, info) VALUES (?, ?, ?, ?, ?)",
            params![monitor_id, result as u8, response_time, time, info],
        )?;

    Ok(())
}
