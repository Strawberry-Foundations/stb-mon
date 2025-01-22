use anyhow::{anyhow, bail};
use std::collections::HashMap;
use std::str::FromStr;
use std::{net::SocketAddr, time::Duration};

use axum::{extract::Query, http::StatusCode};

use crate::monitor::tcp::TcpExpectedResponse;
use crate::{database, monitor::Monitor};

// Query q fields
// ty: service type
// del: check delay in minutes
//
// tcp query
// sa: socket address (host:port)
// exre: expected response (open port: op,
//       bits: bits
//             sh: sent bytes as hex, must be even
//             ex: expected response as string of {0, 1, ?}, must be divisible by 8
// to: timeout in seconds
//
pub async fn route_add_monitor(q: Query<HashMap<String, String>>) -> (StatusCode, String) {
    let Some(Ok(delay_mins)) = q.get("del").map(|del| del.parse::<u16>()) else {
        return (
            StatusCode::BAD_REQUEST,
            "bad or missing `del` (check delay)".to_string(),
        );
    };

    match q.get("ty").map(|s| s.as_str()) {
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "missing param `ty` (service type)".to_string(),
            )
        }
        Some("tcp") => {
            let Some(Ok(socket_addr)) = q.get("socket_addr").map(|sa| SocketAddr::from_str(sa))
            else {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad or missing `sa` (socket address)".to_string(),
                );
            };

            let expected_response = match q.get("exre").map(|s| s.as_str()) {
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "missing param `exre` (expected response)".to_string(),
                    )
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
                    )
                }
            };

            let Some(Ok(timeout_s)) = q.get("del").map(|del| del.parse::<u16>()) else {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad or missing `to` (timeout)".to_string(),
                );
            };

            if let Err(e) = database::monitor::add_monitor(
                Monitor::Tcp {
                    addr: socket_addr,
                    expected: expected_response,
                    timeout: Duration::from_secs(timeout_s as _),
                },
                delay_mins,
            )
            .await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to add monitor to database: {e}"),
                );
            }
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "bad param `ty` (service type), must be one of: {tcp}".to_string(),
            )
        }
    };
    (StatusCode::OK, "Monitor was added".to_string())
}
