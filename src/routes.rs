use anyhow::{anyhow, bail};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;
use std::{net::SocketAddr, time::Duration};

use axum::{extract::Query, http::StatusCode};

use crate::config::CONFIG;
use crate::database::monitor::get_by_id;
use crate::monitor::tcp::TcpExpectedResponse;
use crate::{checker, database, monitor::MonitorData};

pub async fn favicon_route() -> (HeaderMap, Vec<u8>) {
    let hm = HeaderMap::from_iter(vec![(
        CONTENT_TYPE,
        HeaderValue::from_str("image/png").unwrap(),
    )]);
    let img = include_bytes!("../static/favicon.ico").to_vec();
    (hm, img)
}

pub async fn indexjs_route() -> String {
    include_str!("../static/index.js").to_string()
}

// Query q fields
// ty: service type
// in: check interval in minutes
//
// tcp query
// sa: socket address (host:port)
// exre: expected response (open port: op,
//       bits: bits
//             sh: sent bytes as hex, must be even
//             ex: expected response as string of {0, 1, ?}, must be divisible by 8
// to: timeout in seconds
//
pub async fn add_monitor_route(q: Query<HashMap<String, String>>) -> (StatusCode, String) {
    let Some(Ok(interval_mins)) = q.get("in").map(|del| del.parse::<u16>()) else {
        return (
            StatusCode::BAD_REQUEST,
            "bad or missing `in` (check interval)".to_string(),
        );
    };

    if interval_mins < 1 || interval_mins > 60 * 24 * 7 /* 7 days */ {
        return (
            StatusCode::BAD_REQUEST,
            "bad param `ìn` (check interval), must be within 1..94080".to_string()
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
                    // stolen from https://play.rust-lang.org/?version=stable&mode=debug&edition=2015&gist=e241493d100ecaadac3c99f37d0f766f
                    fn decode_hex(s: &str) -> anyhow::Result<Vec<u8>> {
                        if s.len() % 2 != 0 {
                            bail!("Odd length")
                        } else {
                            (0..s.len())
                                .step_by(2)
                                .map(|i| {
                                    u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| anyhow!(e))
                                })
                                .collect()
                        }
                    }

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

            let Some(Ok(timeout_s)) = q.get("to").map(|del| del.parse::<u16>()) else {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad or missing param `to` (timeout)".to_string(),
                );
            };

            if timeout_s < 1 || timeout_s > 60 {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad param `to` (timeout), must be within 1..60".to_string()
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
                        format!("Failed to add monitor to database: {e}"),
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

    let mon = get_by_id(id).await.unwrap();
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
