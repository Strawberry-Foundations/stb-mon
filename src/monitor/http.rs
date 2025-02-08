use std::{
    collections::HashMap,
    error::Error,
    time::{Duration, Instant},
};

use adler32::adler32;
use axum::http::{HeaderName, HeaderValue};
use reqwest::{redirect::Policy, StatusCode};
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

use super::MonitorResult;

pub fn parse_codes(val: &str) -> Option<Vec<StatusCode>> {
    let val = val.replace(' ', "");
    let allowed_chars: Vec<char> = "0123456789,-".chars().collect();
    if val.chars().any(|c| !allowed_chars.contains(&c)) {
        return None;
    }

    let mut codes = vec![];
    let parts = val.split(',');
    for part in parts {
        let n_hyp = part.chars().filter(|c| *c == '-').count();
        if n_hyp >= 2 {
            return None;
        }

        if n_hyp == 0 {
            // single-status part
            if let Ok(n) = part.parse::<u16>() {
                codes.push(StatusCode::from_u16(n));
                continue;
            } else {
                return None;
            }
        }

        // range part (x-y)
        let (start, end) = part.split_once('-').unwrap();
        let (Ok(start), Ok(end)) = (start.parse::<u16>(), end.parse::<u16>()) else {
            return None;
        };

        if start > end {
            return None;
        }

        for c in start..=end {
            codes.push(StatusCode::from_u16(c));
        }
    }

    let codes = codes.into_iter().filter_map(Result::ok).collect();
    Some(codes)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Trace,
    Connect,
    Patch,
}

impl HttpMethod {
    pub fn to_reqwest(&self) -> reqwest::Method {
        use reqwest::Method as RM;
        match self {
            Self::Get => RM::GET,
            Self::Post => RM::POST,
            Self::Put => RM::PUT,
            Self::Delete => RM::DELETE,
            Self::Options => RM::OPTIONS,
            Self::Head => RM::HEAD,
            Self::Trace => RM::TRACE,
            Self::Connect => RM::CONNECT,
            Self::Patch => RM::PATCH,
        }
    }

    pub fn from_str(val: &str) -> Option<Self> {
        match val.to_lowercase().as_str() {
            "get" => Some(Self::Get),
            "post" => Some(Self::Post),
            "put" => Some(Self::Put),
            "delete" => Some(Self::Delete),
            "options" => Some(Self::Options),
            "head" => Some(Self::Head),
            "trace" => Some(Self::Trace),
            "connect" => Some(Self::Connect),
            "patch" => Some(Self::Patch),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HeaderHashMap(HashMap<String, String>);

impl HeaderHashMap {
    pub fn try_parse_str(val: &str) -> Option<Self> {
        let mut headers = HashMap::new();
        let lines = val.lines();

        for header in lines {
            let (k, v) = header.split_once(':')?;
            headers.insert(k.trim().to_string(), v.trim().to_string());
        }

        let hhm = Self(headers);
        Self::to_reqwest(&hhm)?;

        Some(hhm)
    }

    pub fn to_reqwest(&self) -> Option<reqwest::header::HeaderMap> {
        let mut hm = reqwest::header::HeaderMap::new();

        for (h, v) in &self.0 {
            let Ok(name) = HeaderName::from_lowercase(h.to_lowercase().as_bytes()) else {
                return None;
            };

            let Ok(value) = HeaderValue::from_str(v) else {
                return None;
            };

            hm.insert(name, value);
        }

        Some(hm)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub headers: HeaderHashMap,
    pub body: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HttpExpectedResponse {
    // The service sent any response
    Any,
    // The service sent a response with a status code
    // looks like this: 200,300-399,400,500-599
    // verified at creation time
    StatusCode(String),
    // The server replies with specified bytes after sending the bytes
    // (status_code, body_a32)
    Response(Option<String>, u32),
}

pub async fn http_service(
    url: &String,
    expected: &HttpExpectedResponse,
    timeout: Duration,
    request_data: &HttpRequest,
) -> MonitorResult {
    let start_time = Instant::now();
    let config = CONFIG.get().unwrap().lock().await;
    let client = reqwest::ClientBuilder::new()
        .redirect(if config.http.follow_redirects {
            let limit = config.http.max_follow_redirects;
            Policy::limited(limit.unwrap().into())
        } else {
            Policy::none()
        })
        .build()
        .unwrap();

    let res = client
        .request(request_data.method.to_reqwest(), url)
        .headers(request_data.headers.to_reqwest().unwrap())
        .body(request_data.body.clone())
        .timeout(timeout)
        .send()
        .await;

    let res = match res {
        Ok(res) => res,
        Err(e) => {
            if e.is_timeout() {
                return MonitorResult::Down(format!("Connection timed out: {:?}", e.source()));
            }
            return MonitorResult::IoError(format!("reqwest threw error: {:?}", e.source()));
        }
    };

    if config.http.fivexx_status_code_down && (500..599).contains(&res.status().as_u16())
    {
        return MonitorResult::Down(format!("Server replied with status {}", res.status()));
    }

    let delta = Instant::now().duration_since(start_time).as_millis();
    match expected {
        HttpExpectedResponse::Any => MonitorResult::Ok(
            delta,
            format!(
                "Server replied with status {} and {} bytes",
                res.status(),
                res.bytes().await.map(|b| b.len()).unwrap_or_default()
            ),
        ),
        HttpExpectedResponse::StatusCode(codes) => {
            let codes = parse_codes(codes).unwrap();
            let status = res.status();
            let Ok(bytes) = res.bytes().await else {
                return MonitorResult::IoError("Failed to parse response bytes".to_string());
            };
            let info = format!(
                "Server replied with status {status} and {} bytes",
                bytes.len(),
            );

            if codes.contains(&status) {
                MonitorResult::Ok(delta, info)
            } else {
                MonitorResult::UnexpectedResponse(delta, info)
            }
        }
        HttpExpectedResponse::Response(code, body_checksum) => {
            let status = res.status();
            let Ok(res_bytes) = res.bytes().await else {
                return MonitorResult::IoError("Failed to parse response bytes".to_string());
            };

            if code
                .clone()
                .is_some_and(|c| !parse_codes(&c).unwrap().contains(&status))
            {
                return MonitorResult::UnexpectedResponse(
                    delta,
                    format!(
                        "Server replied with status {status} and {} bytes",
                        res_bytes.len(),
                    ),
                );
            }

            let body_a32 = adler32(res_bytes.to_vec().as_slice()).unwrap();
            if body_checksum != &body_a32 {
                return MonitorResult::UnexpectedResponse(
                    delta,
                    format!("Body checksum mismatch ({body_checksum} != {body_a32})"),
                );
            }

            MonitorResult::Ok(
                delta,
                format!(
                    "Server replied with status {status} and {} bytes",
                    res_bytes.len(),
                ),
            )
        }
    }
}
