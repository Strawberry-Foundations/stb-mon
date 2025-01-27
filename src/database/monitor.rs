use anyhow::bail;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::params;
use std::collections::HashMap;

use crate::monitor::{Monitor, MonitorData};

use super::DATABASE;

// returns the id of the added monitor
pub async fn add(service_data: MonitorData, interval_mins: u16, service_name: String) -> anyhow::Result<u64> {
    tracing::debug!(
        "Adding monitor - service_data: {service_data:?} | interval_mins: {interval_mins}"
    );
    let service_data = rmp_serde::to_vec(&service_data)?;
    let db = DATABASE.lock().await;
    db.execute(
        "INSERT INTO monitors (serviceDataMp, intervalMins, serviceName) VALUES (?, ?, ?)",
        params![service_data, interval_mins, service_name],
    )?;

    let id = db.query_row(
        "SELECT id FROM monitors ORDER BY id DESC LIMIT 1",
        [],
        |r| r.get(0),
    )?;

    Ok(id)
}

pub async fn get_by_id(id: u64) -> Option<Monitor> {
    let mon: Monitor = DATABASE
        .lock()
        .await
        .query_row(
            "SELECT serviceDataMp, intervalMins, enabled, serviceName FROM monitors WHERE id = ?",
            [id],
            |r| {
                let service_data: Vec<u8> = r.get(0).unwrap();
                let service_data: MonitorData = rmp_serde::from_slice(&service_data).unwrap();
                let interval_mins: u64 = r.get(1).unwrap();
                let enabled: bool = r.get(2).unwrap();
                let service_name: String = r.get(3).unwrap();

                let mon = Monitor {
                    service_data,
                    service_name,
                    interval_mins,
                    enabled,
                };
                Ok(mon)
            },
        )
        .ok()?;

    Some(mon)
}

pub async fn get_all(enabled_only: bool) -> anyhow::Result<HashMap<u64, Monitor>> {
    let lock = DATABASE.lock().await;
    let mut stmt = lock.prepare(&format!(
        "SELECT id, serviceDataMp, intervalMins, enabled, serviceName FROM monitors {}",
        if enabled_only {
            "WHERE enabled = 1"
        } else {
            ""
        }
    ))?;
    let res: HashMap<u64, Monitor> = stmt
        .query([])?
        .map(|r| {
            let id: u64 = r.get(0).unwrap();
            let service_data: Vec<u8> = r.get(1).unwrap();
            let service_data: MonitorData = rmp_serde::from_slice(&service_data).unwrap();
            let interval_mins: u64 = r.get(2).unwrap();
            let enabled: bool = r.get(3).unwrap();
            let service_name: String = r.get(4).unwrap();


            let mon = Monitor {
                service_data,
                service_name,
                interval_mins,
                enabled,
            };
            Ok((id, mon))
        })
        .collect()
        .unwrap();
    Ok(res)
}

pub async fn util_delete(id: u64) -> anyhow::Result<()> {
    let affected = DATABASE
        .lock()
        .await
        .execute("DELETE FROM monitors WHERE id = ?", [id])?;
    if affected == 0 {
        bail!("No such monitor")
    }
    DATABASE
        .lock()
        .await
        .execute("DELETE FROM records WHERE monitorId = ?", [id])?;
    Ok(())
}

pub async fn toggle(id: u64) -> anyhow::Result<bool> {
    let enabled = match is_enabled(id).await {
        Ok(e) => e,
        Err(rusqlite::Error::QueryReturnedNoRows) => bail!("No such monitor"),
        Err(e) => bail!(e),
    };

    DATABASE
        .lock()
        .await
        .execute("UPDATE monitors SET enabled = ? WHERE id = ?", params![
            !enabled, id
        ])?;

    Ok(!enabled)
}

pub async fn is_enabled(id: u64) -> rusqlite::Result<bool> {
    let enabled: bool = DATABASE.lock().await.query_row(
        "SELECT enabled FROM monitors WHERE id = ?",
        [id],
        |r| {
            let enabled: bool = r.get(0).unwrap();

            Ok(enabled)
        },
    )?;

    Ok(enabled)
}
