use crate::time_util::current_unix_time;
use rusqlite::params;

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

pub async fn add(
    result: RecordResult,
    response_time: Option<u64>,
    monitor_id: i32,
    info: String,
) -> anyhow::Result<()> {
    tracing::debug!(
        "Adding record - result: {result:?} | response_time: {response_time:?} | monitor_id: {monitor_id} | info: {info}"
    );

    DATABASE
        .get()
        .ok_or(anyhow::anyhow!("Failed to get database"))?
        .lock()
        .await
        .execute(
            "INSERT INTO records (monitorId, result, responseDeltaMs, checkedAt, info) VALUES (?, ?, ?, ?, ?)",
            params![monitor_id, result as u8, response_time, current_unix_time(), info],
        )?;

    Ok(())
}

pub async fn util_last_record(mon_id: i32) -> anyhow::Result<u64> {
    Ok(DATABASE
        .get()
        .ok_or(anyhow::anyhow!("Failed to get database"))?
        .lock()
        .await
        .query_row(
            "SELECT checkedAt FROM records WHERE id = ? ORDER BY checkedAt DESC LIMIT 1",
            [mon_id],
            |r| r.get(0),
        )?)
}
