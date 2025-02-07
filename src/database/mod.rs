use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod monitor;
pub mod record;
pub mod session;

static DATABASE_PATH: &str = "./stbmon.sqlite";

lazy_static! {
    pub static ref DATABASE: Arc<Mutex<Connection>> = {
        let database = Connection::open(DATABASE_PATH).expect("Failed to open database");
        database
            .execute(
                r"
        CREATE TABLE IF NOT EXISTS monitors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            serviceDataMp BLOB NOT NULL,
            intervalMins INTEGER NOT NULL,
            enabled BOOLEAN DEFAULT 1,
            serviceName VARCHAR NOT NULL,
            timeoutSecs INTEGER NOT NULL
        )
        ",
                [],
            )
            .expect("Failed to run query");

        database
            .execute(
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
            )
            .expect("Failed to run query");

        database
            .execute(
                r"
        CREATE TABLE IF NOT EXISTS sessions (
            token VARCHAR PRIMARY KEY,
            expiresAt INTEGER
        );
        ",
                [],
            )
            .expect("Failed to run query");

        Arc::new(Mutex::new(database))
    };
}
