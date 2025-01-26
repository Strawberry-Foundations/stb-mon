use std::{collections::HashMap, str::FromStr, time::Duration};

use axum::http::{HeaderName, HeaderValue};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use super::MonitorResult;

pub fn parse_codes(val: &str) -> Option<Vec<StatusCode>> {
    let val = val.replace(' ', "");
    let allowed_chars: Vec<char> = "0123456789,-".chars().collect();
    if val.chars().any(|c| !allowed_chars.contains(&c)) {
        return None;
    }

    let mut codes = vec![];
    let parts = val.split(",");
    for part in parts {
        let n_hypen = part.chars().filter(|c| *c == '-').count();
        if n_hypen >= 2 {
            return None;
        }
        if n_hypen == 0 {
            // single-status part
            if let Ok(n) = part.parse::<u16>() {
                codes.push(StatusCode::from_u16(n));
            } else {
                return None;
            }
        }
        // range part (x-y)
        let (start, end) = part.split_once("-").unwrap();
        let (Ok(start), Ok(end)) = (start.parse::<u16>(), end.parse::<u16>()) else {
            return None;
        };

        if end > start {
            return None;
        }

        for c in start..=end {
            codes.push(StatusCode::from_u16(c));
        }
    }

    let codes = codes.into_iter().filter_map(|r| r.ok()).collect();
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
    // (status_code, text_received)
    Response(Option<u16>, String),
}

pub async fn http_service(
    url: &String,
    expected: &HttpExpectedResponse,
    timeout: Duration,
    request_data: &HttpRequest,
) -> MonitorResult {
    // TODO: implement http service
    MonitorResult::Ok(0, "this service type is not implemented".to_string())
}
