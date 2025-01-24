use std::time::Duration;

use rusqlite::fallible_iterator::FallibleIterator;

use crate::time_util::current_unix_time;
use crate::{
    database::{
        DATABASE,
        record::{self, RecordResult},
    },
    monitor::{MonitorData, MonitorResult},
};

static CHECK_INTERVAL: Duration = Duration::from_secs(5);

async fn get_pending_checks() -> Vec<(i32, MonitorData)> {
    let lock = DATABASE.get().unwrap().lock().await;
    let mut stmt = lock
        .prepare(
            /*
            SELECT id,
                   checkedAt
            FROM monitors
            INNER JOIN records ON records.monitorId = monitors.id
              AND records.checkedAt =
                (SELECT checkedAt
                 FROM records
                 WHERE id = monitors.id
                 ORDER BY checkedAt DESC
                 LIMIT 1)
            WHERE checkedAt > ?
            */
            r"SELECT id, checkedAt FROM monitors INNER JOIN records ON records.monitorId = monitors.id AND records.checkedAt = (SELECT checkedAt FROM records WHERE id = monitors.id ORDER BY checkedAt DESC LIMIT 1) WHERE checkedAt > ?",
        )
        .unwrap();

    stmt.query([current_unix_time()])
        .unwrap()
        .map(|r| {
            let id: i32 = r.get(0).unwrap();
            let mp_bytes: Vec<u8> = r.get(1).unwrap();
            let service_data = rmp_serde::from_slice::<MonitorData>(&mp_bytes).unwrap();
            Ok((id, service_data))
        })
        .collect()
        .unwrap()
}

pub async fn add_result(res: MonitorResult, mon_id: i32) -> anyhow::Result<()> {
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
        let mons = get_pending_checks().await;
        for (id, mon) in mons {
            tracing::info!("Running monitor {id}");
            let res = mon.run().await;
            let res = add_result(res, id).await;

            if let Err(e) = res {
                tracing::error!("Failed to add result of monitor {id} to database: {e}");
                continue; // monitor did not run, so we don't update the nextCheck of it
            }
        }
        tokio::time::sleep(CHECK_INTERVAL).await;
    }
}
