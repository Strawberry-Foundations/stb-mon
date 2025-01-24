use anyhow::bail;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use rusqlite::params;

use crate::monitor::{Monitor, MonitorData};

use super::DATABASE;

// returns the id of the added monitor
pub async fn add(service_data: MonitorData, interval_mins: u16) -> anyhow::Result<i32> {
    tracing::debug!(
        "Adding monitor - service_data: {service_data:?} | interval_mins: {interval_mins}"
    );
    let service_data = rmp_serde::to_vec(&service_data)?;
    let db = DATABASE
        .get()
        .ok_or(anyhow::anyhow!("Failed to get database"))?
        .lock()
        .await;
    db.execute(
        "INSERT INTO monitors (serviceDataMp, intervalMins) VALUES (?, ?)",
        params![service_data, interval_mins],
    )?;

    let id = db.query_row(
        "SELECT id FROM monitors ORDER BY id DESC LIMIT 1",
        [],
        |r| r.get(0),
    )?;

    Ok(id)
}

pub async fn get_by_id(id: i32) -> Option<Monitor> {
    let mon: Monitor = DATABASE
        .get()?
        .lock()
        .await
        .query_row(
            "SELECT serviceDataMp, intervalMins, enabled FROM monitors WHERE id = ?",
            [id],
            |r| { 
                let service_data: Vec<u8> = r.get(0).unwrap();
                let service_data: MonitorData = rmp_serde::from_slice(&service_data).unwrap();
                let interval_mins: i32 = r.get(1).unwrap();
                let enabled: bool = r.get(2).unwrap();
                
                let mon = Monitor {
                    service_data,
                    interval_mins,
                    enabled
                };
                Ok(mon)
            },
        )
        .ok()?;

    Some(mon)
}

pub async fn get_all() -> anyhow::Result<Vec<(i32, Monitor)>> {
    let lock = DATABASE.get().unwrap().lock().await;
    let mut stmt = lock.prepare("SELECT id, serviceDataMp, intervalMins, enabled FROM monitors")?;
    let res = stmt.query([])?
        .map(|r| {
            let id: i32 = r.get(0).unwrap();
            let service_data: Vec<u8> = r.get(1).unwrap();
            let service_data: MonitorData = rmp_serde::from_slice(&service_data).unwrap();
            let interval_mins: i32 = r.get(2).unwrap();
            let enabled: bool = r.get(3).unwrap();
            
            let mon = Monitor {
                service_data,
                interval_mins,
                enabled,
            };
            Ok((id, mon))
        })
        .collect()
        .unwrap();
    
    Ok(res)
}
