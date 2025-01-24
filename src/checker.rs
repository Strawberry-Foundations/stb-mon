use std::collections::HashMap;
use std::time::Duration;

use rusqlite::fallible_iterator::FallibleIterator;

use crate::time_util::current_unix_time;
use crate::{
    database,
    database::{
        DATABASE,
        record::{self, RecordResult},
    },
    monitor::MonitorResult,
};

static CHECK_INTERVAL: Duration = Duration::from_secs(5);

async fn get_records() -> Vec<(u64, u64)> {
    let lock = DATABASE.lock().await;
    let mut stmt = lock
        .prepare(
            r"SELECT monitorId, MAX(checkedAt) as maxCheckedAt FROM records GROUP BY monitorId",
        )
        .unwrap();

    let last_records: Vec<(u64, u64)> = stmt
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
    let mons = database::monitor::get_all().await.unwrap();
    let now = current_unix_time();
    for (id, time) in last_records {
        let mon = mons.get(&id).unwrap();
        if time + 60 * mon.interval_mins < now {
            let res = mon.service_data.run().await;
            add_result(res, id).await.unwrap();
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
        MonitorResult::ConnectionTimeout => {
            record::add(
                RecordResult::Down,
                None,
                mon_id,
                "The server did not reply within the timeout".to_string(),
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
