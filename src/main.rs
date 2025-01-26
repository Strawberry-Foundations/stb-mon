use crate::config::CONFIG;

use anyhow::Context;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use checker::checker_thread;
use std::env;

mod api;
mod checker;
mod config;
mod database;
mod monitor;
mod routes;
mod templates;
mod time_util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "./stbmon.toml".to_string());

    tracing::info!("Loading config");

    config::init_config(config_path)
        .await
        .context("Failed to initialize config")?;

    let app = Router::new()
        .route("/", get(templates::index_template))
        .route("/admin", get(templates::admin_template))
        .route("/favicon.ico", get(routes::favicon_route))
        .route("/index.js", get(routes::indexjs_route))
        .route("/admin.js", get(routes::adminjs_route))
        .route("/api/monitors/{id}", delete(api::delete_monitor_route))
        .route("/api/monitors/{id}/toggle", put(api::toggle_monitor))
        .route("/api/monitors", put(api::add_monitor_route))
        .route("/api/create_session", post(api::create_session_route));

    let bind_addr = CONFIG.get().unwrap().lock().await.bind_addr;

    tracing::info!("Binding HTTP server to http://{bind_addr}");

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .context("Failed to start web server")?;

    tokio::task::spawn(checker_thread());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
