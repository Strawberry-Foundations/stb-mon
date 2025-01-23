use std::time::{Duration, UNIX_EPOCH};

use rusqlite::fallible_iterator::FallibleIterator;

use crate::{
    database::{
        record::{self, RecordResult},
        DATABASE,
    },
    monitor::{Monitor, MonitorResult},
};

static CHECK_DELAY: Duration = Duration::from_secs(5);

async fn get_pending_checks() -> Vec<(i32, Monitor)> {
    let time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let Some(db) = DATABASE.get() else {
        return vec![];
    };
    let lock = db.lock().await;
    let mut stmt = lock
        .prepare(
            "SELECT id, serviceDataMp FROM monitors WHERE (nextCheck <= ? OR nextCheck IS NULL)",
        )
        .unwrap();

    stmt.query([time])
        .unwrap()
        .map(|r| {
            let id: i32 = r.get(0).unwrap();
            let mp_bytes: Vec<u8> = r.get(1).unwrap();
            Ok((id, rmp_serde::from_slice::<Monitor>(&mp_bytes).unwrap()))
        })
        .collect()
        .unwrap()
}

pub async fn checker_thread() {
    loop {
        tracing::debug!("Getting pending checks");
        let mons = get_pending_checks().await;
        for (id, mon) in mons {
            tracing::info!("Running monitor {id}");
            let res = mon.run().await;
            let res = match res {
                MonitorResult::Ok(response_time_ms, info) => {
                    record::add_record(RecordResult::Ok, Some(response_time_ms as _), id, info)
                        .await
                }
                MonitorResult::UnexpectedResponse(response_time_ms, info) => {
                    record::add_record(
                        RecordResult::Unexpected,
                        Some(response_time_ms as _),
                        id,
                        info,
                    )
                    .await
                }
                MonitorResult::ConnectionTimeout => {
                    record::add_record(
                        RecordResult::Down,
                        None,
                        id,
                        "The server did not reply within the timeout".to_string(),
                    )
                    .await
                }
                MonitorResult::IoError(err) => {
                    record::add_record(RecordResult::Err, None, id, err).await
                }
            };

            if let Err(e) = res {
                tracing::error!("Failed to add result of monitor {id} to database: {e}")
            }
        }
        tokio::time::sleep(CHECK_DELAY).await;
    }
}
