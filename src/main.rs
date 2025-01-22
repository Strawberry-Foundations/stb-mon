use std::env;

use crate::config::lock_config;
use crate::database::initialize_database;
use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use checker::checker_thread;

mod checker;
mod config;
mod database;
mod monitor;
mod octet;
mod routes;
mod templates;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    initialize_database()?;
    tracing_subscriber::fmt::init();
    tokio::spawn(checker_thread());

    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "./stbmon.toml".to_string());

    tracing::info!("Loading config");

    config::init_config(config_path)
        .await
        .context("Failed to initialize config")?;

    let app = Router::new()
        .route("/", get(templates::index))
        .route("/api/add_monitor", post(routes::route_add_monitor));

    let bind_addr = lock_config().await.bind_addr;
    tracing::info!("Binding HTTP server to {bind_addr}");
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .context("Failed to start web server")?;
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
