use anyhow::bail;
use rusqlite::Connection;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

pub mod monitor;
pub mod records;

static DATABASE_PATH: &'static str = "./stbmon.sqlite";
pub static DATABASE: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

pub fn initialize_database() -> anyhow::Result<()> {
    if DATABASE.get().is_some() {
        bail!("Database already initialized");
    }
    let database = Connection::open(DATABASE_PATH)?;
    database.execute(
        r"
    CREATE TABLE IF NOT EXISTS monitors (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        serviceDataMp BLOB NOT NULL,
        delayMins INTEGER NOT NULL,
        nextCheck INTEGER,
        lastCheck INTEGER
    )
    ",
        [],
    )?;
    database.execute(
        r"
    CREATE TABLE IF NOT EXISTS records (
        monitorId INTEGER NOT NULL,
        result INTEGER NOT NULL,
        responseDeltaMs INTEGER,
        checkedAt INTEGER NOT NULL,
        info VARCHAR
    );
    ",
        [],
    )?;
    DATABASE.set(Arc::new(Mutex::new(database))).unwrap();
    Ok(())
}
