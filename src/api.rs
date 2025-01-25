use std::{collections::HashMap, net::SocketAddr, str::FromStr, time::Duration};

use axum::{
    extract::{Path, Query},
    http::StatusCode,
};
use axum_extra::extract::CookieJar;

use crate::{
    checker,
    config::CONFIG,
    database,
    monitor::{MonitorData, tcp::TcpExpectedResponse},
};

// Query q fields
// ty: service type
// in: check interval in minutes
//
// tcp query
// sa: socket address (host:port)
// exre: expected response
//       open port: op,
//       bytes: bytes as hex
//             sh: sent bytes as hex, must be even
//             ex: expected response as string of hex + ?, must be divisible by 2
// to: timeout in seconds
//
pub async fn add_monitor_route(
    q: Query<HashMap<String, String>>,
    cookies: CookieJar,
) -> (StatusCode, String) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or(false),
    };
    if !is_logged_in {
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized (set token cookie to log in)".to_string(),
        );
    }

    let Some(Ok(interval_mins)) = q.get("in").map(|i: &String| i.parse::<u16>()) else {
        return (
            StatusCode::BAD_REQUEST,
            "bad or missing `in` (check interval)".to_string(),
        );
    };

    if interval_mins < 1 || interval_mins > 60 * 24 * 7
    /* 7 days */
    {
        return (
            StatusCode::BAD_REQUEST,
            "bad param `ìn` (check interval), must be within 1..94080".to_string(),
        );
    }
    let id = match q.get("ty").map(|s| s.as_str()) {
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "missing param `ty` (service type)".to_string(),
            );
        }
        Some("tcp") => {
            let Some(Ok(socket_addr)) = q.get("sa").map(|sa| SocketAddr::from_str(sa)) else {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad or missing param `sa` (socket address)".to_string(),
                );
            };

            let expected_response = match q.get("exre").map(|s| s.as_str()) {
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "missing param `exre` (expected response)".to_string(),
                    );
                }
                Some("op") => TcpExpectedResponse::OpenPort,
                Some("bits") => {
                    todo!()
                }
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "bad param `exre` (expected response), must be one of: {op, bits}"
                            .to_string(),
                    );
                }
            };

            let Some(Ok(timeout_s)) = q.get("to").map(|to| to.parse::<u16>()) else {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad or missing param `to` (timeout)".to_string(),
                );
            };

            if timeout_s < 1 || timeout_s > 60 {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad param `to` (timeout), must be within 1..60".to_string(),
                );
            }

            match database::monitor::add(
                MonitorData::Tcp {
                    addr: socket_addr,
                    expected: expected_response,
                    timeout: Duration::from_secs(timeout_s as _),
                },
                interval_mins,
            )
            .await
            {
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to add monitor: {e}"),
                    );
                }
                Ok(id) => id,
            }
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "bad param `ty` (service type), must be one of: {tcp}".to_string(),
            );
        }
    };

    let mon = database::monitor::get_by_id(id).await.unwrap();
    let res = mon.service_data.run().await;
    checker::add_result(res, id).await.unwrap();

    (StatusCode::OK, "Monitor was added".to_string())
}

pub async fn create_session_route(q: Query<HashMap<String, String>>) -> (StatusCode, String) {
    let Some(password) = q.get("pw") else {
        return (
            StatusCode::BAD_REQUEST,
            "missing param `pw` (password)".to_string(),
        );
    };

    if !CONFIG.get().unwrap().lock().await.check_password(password) {
        return (StatusCode::UNAUTHORIZED, "wrong password".to_string());
    };

    let token = match database::session::create().await {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to add session to database: {e}"),
            );
        }
    };

    (StatusCode::OK, token)
}

pub async fn delete_monitor_route(id: Path<u64>, cookies: CookieJar) -> (StatusCode, String) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or(false),
    };
    if !is_logged_in {
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized (set `token` cookie to log in)".to_string(),
        );
    };

    if let Err(e) = database::monitor::util_delete_monitor(*id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to remove monitor: {e}"),
        );
    }

    (StatusCode::OK, "Monitor was deleted".to_string())
}
