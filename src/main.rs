use crate::config::CONFIG;

use anyhow::Context;
use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use checker::checker_thread;
use database::DATABASE;
use rusqlite::fallible_iterator::FallibleIterator;
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

    tracing::debug!("Fixing monitors");
    match fix_no_records().await {
        Ok(fixed) if fixed > 0 => {
            tracing::info!("Fixed {fixed} monitors!");
        }
        Err(e) => tracing::error!("Failed to fix monitors: {e}"),
        _ => {}
    };

    let app = Router::new()
        .route("/", get(templates::index_template))
        .route("/admin", get(templates::admin_template))
        .route("/monitor/{id}", get(templates::monitor_template))
        .route("/static/{*path}", get(routes::static_route))
        .route("/api/monitors/{id}", delete(api::delete_monitor_route))
        .route("/api/monitors/{id}/toggle", patch(api::toggle_monitor))
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

// create records for monitors with no records
async fn fix_no_records() -> anyhow::Result<usize> {
    let lock = DATABASE.lock().await;
    let mut stmt = lock.prepare("SELECT id FROM monitors WHERE id NOT IN (SELECT monitorId FROM records)")?;

    let ids: Vec<u64> = stmt.query([])?
        .map(|r| Ok(r.get::<_, u64>(0).unwrap()))
        .collect()
        .unwrap();
    drop(stmt);
    drop(lock);

    let mut fixed = 0;
    for id in ids {
        tracing::debug!("Fixing monitor {id}");
        let mon = database::monitor::get_by_id(id).await.unwrap();
        let result = mon.service_data.run(mon.timeout_secs).await;
        database::record::util_add_result(result, id).await?;
        fixed += 1;

    }

    Ok(fixed)
}