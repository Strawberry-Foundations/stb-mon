use crate::time_util::current_unix_time;
use rusqlite::params;

use super::DATABASE;

pub struct MonitorRecord {
    pub time_checked: u64,
    pub result: RecordResult,
    // None = timeout/error
    pub response_time_ms: Option<u64>,
    pub monitor_id: u64,
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

impl From<u8> for RecordResult {
    fn from(value: u8) -> Self {
        match value {
            0 => RecordResult::Ok,
            1 => RecordResult::Unexpected,
            2 => RecordResult::Down,
            3 => RecordResult::Err,
            _ => unreachable!(),
        }
    }
}

pub async fn add(
    result: RecordResult, // None results are the ones created when a monitor is added
    response_time: Option<u64>,
    monitor_id: u64,
    info: String,
) -> anyhow::Result<()> {
    tracing::debug!(
        "Adding record - result: {result:?} | response_time: {response_time:?} | monitor_id: {monitor_id} | info: {info}"
    );

    DATABASE
        .lock()
        .await
        .execute(
            "INSERT INTO records (monitorId, result, responseDeltaMs, checkedAt, info) VALUES (?, ?, ?, ?, ?)",
            params![monitor_id, result as u8, response_time, current_unix_time(), info],
        )?;

    Ok(())
}

pub async fn util_last_record(mon_id: u64) -> anyhow::Result<MonitorRecord> {
    Ok(DATABASE
        .lock()
        .await
        .query_row(
            "SELECT monitorId, result, responseDeltaMs, checkedAt, info FROM records WHERE monitorId = ? ORDER BY checkedAt DESC LIMIT 1",
            [mon_id],
            |r| {
                let monitor_id: u64 = r.get(0).unwrap();
                let result: u8 = r.get(1).unwrap();
                let result = RecordResult::from(result);
                let response_time_ms: Option<u64> = r.get(2).unwrap();
                let time_checked: u64 = r.get(3).unwrap();
                let info: String = r.get(4).unwrap();

                let rec = MonitorRecord {
                    time_checked,
                    result,
                    response_time_ms,
                    monitor_id,
                    info
                };
                Ok(rec)
            },
        )?)
}
