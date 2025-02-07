use anyhow::bail;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub instance_name: String,
    #[serde(rename = "password_sha256")]
    pub password: String,
    pub allow_guest: bool,
    pub http: ConfigHttp,
}

#[derive(Deserialize, Debug)]
pub struct ConfigHttp {
    #[serde(rename = "5xx_status_code_down")]
    pub fivexx_status_code_down: bool,
}

impl Config {
    pub fn check_password(&self, inp: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(inp);
        let hash = hasher.finalize();

        hex::encode(hash) == self.password.to_ascii_lowercase()
    }
}

pub static CONFIG: OnceLock<Arc<Mutex<Config>>> = OnceLock::new();

pub async fn init_config(path: String) -> anyhow::Result<()> {
    if CONFIG.get().is_some() {
        bail!("Config already initialized");
    }

    let file = fs::read_to_string(path).await?;
    let config = toml::from_str::<Config>(&file)?;

    if config.password.len() != 256 / 4 || config.password.chars().any(|c| !c.is_ascii_hexdigit()) {
        bail!("Password is not a valid SHA256")
    }
    
    CONFIG.set(Arc::new(Mutex::new(config))).unwrap();

    Ok(())
}
