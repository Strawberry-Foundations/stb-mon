use crate::time_util::current_unix_time;
use crate::{
    database,
    database::{
        DATABASE,
        record::{self, RecordResult},
    },
    monitor::MonitorResult,
};

use rusqlite::fallible_iterator::FallibleIterator;
use std::collections::HashMap;
use std::time::Duration;

static CHECK_INTERVAL: Duration = Duration::from_secs(5);

async fn get_records() -> HashMap<u64, u64> {
    let lock = DATABASE.lock().await;
    let mut stmt = lock
        .prepare(
            r"SELECT monitorId, MAX(checkedAt) as maxCheckedAt FROM records GROUP BY monitorId",
        )
        .unwrap();

    let last_records: HashMap<u64, u64> = stmt
        .query([])
        .unwrap()
        .map(|r| {
            let monitor_id: u64 = r.get(0).unwrap();
            let checked_at: u64 = r.get(1).unwrap();

            Ok((monitor_id, checked_at))
        })
        .collect()
        .unwrap();

    last_records
}

async fn run_pending_checks() {
    let last_records = get_records().await;
    let mons = database::monitor::get_all(true).await.unwrap();
    let now = current_unix_time();
    for (mon_id, mon) in mons {
        let Some(last_record) = last_records.get(&mon_id) else {
            continue;
        };
        if last_record + 60 * mon.interval_mins < now {
            let res = mon.service_data.run().await;
            add_result(res, mon_id).await.unwrap();
        }
    }
}

pub async fn add_result(res: MonitorResult, mon_id: u64) -> anyhow::Result<()> {
    match res {
        MonitorResult::Ok(response_time_ms, info) => {
            record::add(RecordResult::Ok, Some(response_time_ms as _), mon_id, info).await
        }
        MonitorResult::UnexpectedResponse(response_time_ms, info) => {
            record::add(
                RecordResult::Unexpected,
                Some(response_time_ms as _),
                mon_id,
                info,
            )
            .await
        }
        MonitorResult::Down(is_conn_refused) => {
            let info = if is_conn_refused {
                "Server refused the connection"
            } else {
                "Server did not reply within the timeout"
            }.to_string();
            record::add(
                RecordResult::Down,
                None,
                mon_id,
                info,
            )
            .await
        }
        MonitorResult::IoError(err) => record::add(RecordResult::Err, None, mon_id, err).await,
    }
}

pub async fn checker_thread() {
    loop {
        tracing::debug!("Getting pending checks");
        run_pending_checks().await;

        tokio::time::sleep(CHECK_INTERVAL).await;
    }
}
