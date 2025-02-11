use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use axum::{
    extract::{Path, Query},
    http::StatusCode,
};
use axum_extra::extract::CookieJar;
use base64::{prelude::BASE64_STANDARD, Engine};
use url::Url;

use crate::{
    config::CONFIG,
    database,
    monitor::{
        http::{self as http_mon, HeaderHashMap, HttpExpectedResponse, HttpMethod, HttpRequest},
        tcp::TcpExpectedResponse,
        MonitorData,
    },
};

// Query q fields
// ty: service type
// in: check interval in minutes
// to: timeout in seconds
// na: service name / description, only used for the frontend (empty if none given)
//
// tcp query
// sa: socket address (host:port)
// exre: expected response
//       open port: op,
//       bytes: bytes as hex
//         sh: sent bytes as hex, must be even
//         ex: expected response as string of hex + ?, must be divisible by 2
//
// http query
// url: (http(s)://example.com(:port)/(path))
// exre: expected response
//       any: any response
//       sc: response with status code
//          co: status code range (200-299,301,404-410)
//       res: response with specific code and body
//          co (opt): status codes
//          bch: response body adler32 hash
// met: http method, must be one of {get, post, put, delete, options, head, trace, connect, patch} (GET if not given)
// hds: header map, looks like this: content-type:application/json\naccept:*/* (empty if not given)
// body: base64 encoded request body (empty if none given)
pub async fn add_monitor_route(
    q: Query<HashMap<String, String>>,
    cookies: CookieJar,
) -> (StatusCode, String) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or_default(),
    };
    if !is_logged_in {
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized (set `token` cookie to log in)".to_string(),
        );
    }

    let Some(Ok(interval_mins)) = q.get("in").map(|i: &String| i.parse::<u16>()) else {
        return (
            StatusCode::BAD_REQUEST,
            "bad or missing `in` (check interval)".to_string(),
        );
    };

    let Some(Ok(timeout_s)) = q.get("to").map(|to| to.parse::<u16>()) else {
        return (
            StatusCode::BAD_REQUEST,
            "bad or missing param `to` (timeout)".to_string(),
        );
    };

    let service_name = q.get("na").cloned().unwrap_or_default();

    if !(1..=60 * 24 * 7).contains(&interval_mins)
    /* 7 days */
    {
        return (
            StatusCode::BAD_REQUEST,
            "bad param `ìn` (check interval), must be within 1..94080".to_string(),
        );
    }

    if !(1..=60).contains(&timeout_s) {
        return (
            StatusCode::BAD_REQUEST,
            "bad param `to` (timeout), must be within 1..60".to_string(),
        );
    }

    let id = match q.get("ty").map(String::as_str) {
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

            let expected_response = match q.get("exre").map(String::as_str) {
                Some("op") => TcpExpectedResponse::OpenPort,
                Some("bytes") => {
                    todo!()
                }
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "missing param `exre` (expected response)".to_string(),
                    );
                }
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "bad param `exre` (expected response), must be one of: {op, bits}"
                            .to_string(),
                    );
                }
            };

            match database::monitor::add(
                MonitorData::Tcp {
                    addr: socket_addr,
                    expected: expected_response,
                },
                interval_mins,
                service_name,
                timeout_s,
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
        Some("http") => {
            let Some(Ok(url)) = q.get("url").map(|u| Url::parse(u)) else {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad param `url`, failed to parse valid url".to_string(),
                );
            };
            let url = url.to_string();

            let expected_response = match q.get("exre").map(String::as_str) {
                Some("any") => HttpExpectedResponse::Any,
                Some("sc") => {
                    let Some(codes) = q.get("co") else {
                        return (
                            StatusCode::BAD_REQUEST,
                            "missing param `co` (status codes)".to_string(),
                        );
                    };
                    if codes.len() > 48 {
                        return (
                            StatusCode::BAD_REQUEST,
                            "bad param `co` (status codes), must be at most 48 characters long"
                                .to_string(),
                        );
                    }
                    if http_mon::parse_codes(codes).is_none() {
                        return (
                            StatusCode::BAD_REQUEST,
                            "bad param `co` (status codes), failed to parse".to_string(),
                        );
                    }
                    HttpExpectedResponse::StatusCode(codes.to_string())
                }
                Some("res") => {
                    let codes = if let Some(codes) = q.get("co") {
                        if codes.len() > 48 {
                            return (
                                StatusCode::BAD_REQUEST,
                                "bad param `co` (status codes), must be at most 48 characters long"
                                    .to_string(),
                            );
                        }
                        if http_mon::parse_codes(codes).is_none() {
                            return (
                                StatusCode::BAD_REQUEST,
                                "bad param `co` (status codes), failed to parse".to_string(),
                            );
                        }
                        Some(codes.to_string())
                    } else {
                        None
                    };

                    let Some(Ok(body_checksum)) = q.get("bch").map(|bc| bc.parse::<u32>()) else {
                        return (
                            StatusCode::BAD_REQUEST,
                            "bad or missing param `bch` (body adler32 checksum)".to_string(),
                        );
                    };

                    HttpExpectedResponse::Response(codes, body_checksum)
                }
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "missing param `exre` (expected response)".to_string(),
                    );
                }
                _ => {
                    return (
                        StatusCode::BAD_GATEWAY,
                        "bad param `exre` (expected response), must be one of {res, sc}"
                            .to_string(),
                    );
                }
            };

            let method = if let Some(method) = q.get("met") {
                let Some(method) = HttpMethod::from_str(method) else {
                    return (
                        StatusCode::BAD_REQUEST,
                        "bad param `met`, must be one of {get, post, put, delete, options, head, trace, connect, patch}".to_string(),
                    );
                };

                method
            } else {
                HttpMethod::default()
            };

            let headers = if let Some(headers) = q.get("hds") {
                if headers.len() > 2048 {
                    return (
                        StatusCode::BAD_REQUEST,
                        "bad param `hds` (headers), must be at most 2048 characters long"
                            .to_string(),
                    );
                }
                let Some(hhm) = HeaderHashMap::try_parse_str(headers) else {
                    return (
                        StatusCode::BAD_REQUEST,
                        "bad param `hds` (headers), failed to parse".to_string(),
                    );
                };

                hhm
            } else {
                HeaderHashMap::default()
            };

            let body = q.get("body").map(String::from).unwrap_or_default();
            let Ok(body) = BASE64_STANDARD.decode(body) else {
                return (
                    StatusCode::BAD_REQUEST,
                    "bad param `body`, failed to decode base64".to_string(),
                );
            };

            match database::monitor::add(
                MonitorData::Http {
                    url,
                    expected: expected_response,
                    request: HttpRequest {
                        method,
                        headers,
                        body,
                    },
                },
                interval_mins,
                service_name,
                timeout_s,
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
                "bad param `ty` (service type), must be one of: {tcp, http}".to_string(),
            );
        }
    };

    let mon = database::monitor::get_by_id(id).await.unwrap();
    let res = mon.service_data.run(mon.timeout_secs).await;
    database::record::util_add_result(res, id).await.unwrap();

    (StatusCode::CREATED, "Monitor was added".to_string())
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
            .unwrap_or_default(),
    };
    if !is_logged_in {
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized (set `token` cookie to log in)".to_string(),
        );
    };

    if let Err(e) = database::monitor::util_delete(*id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to remove monitor: {e}"),
        );
    }

    (StatusCode::OK, "Monitor was deleted".to_string())
}

pub async fn toggle_monitor(id: Path<u64>, cookies: CookieJar) -> (StatusCode, String) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or_default(),
    };
    if !is_logged_in {
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized (set `token` cookie to log in)".to_string(),
        );
    };

    let new_status = match database::monitor::toggle(*id).await {
        Ok(true) => "enabled",
        Ok(false) => "disabled",
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to toggle monitor: {e}"),
            );
        }
    };

    (StatusCode::OK, format!("Monitor is now {new_status}"))
}
