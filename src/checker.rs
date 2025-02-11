use crate::database::{self, DATABASE};
use crate::time_util::current_unix_time;

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
            let res = mon.service_data.run(mon.timeout_secs).await;
            database::record::util_add_result(res, mon_id)
                .await
                .unwrap();
        }
    }
}

pub async fn checker_thread() {
    loop {
        tracing::debug!("Getting pending checks");
        run_pending_checks().await;

        tokio::time::sleep(CHECK_INTERVAL).await;
    }
}
