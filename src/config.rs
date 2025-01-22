use anyhow::bail;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use tokio::fs;
use tokio::sync::{Mutex, MutexGuard};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub instance_name: String,
}

pub static CONFIG: OnceLock<Arc<Mutex<Config>>> = OnceLock::new();

pub async fn init_config(path: String) -> anyhow::Result<()> {
    if CONFIG.get().is_some() {
        bail!("Config already initialized");
    }

    let file = fs::read_to_string(path).await?;
    let config = toml::from_str::<Config>(&file)?;
    CONFIG.set(Arc::new(Mutex::new(config))).unwrap();

    Ok(())
}

pub async fn lock_config<'a>() -> MutexGuard<'a, Config> {
    CONFIG.get().unwrap().lock().await
}
