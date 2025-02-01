use std::{collections::HashMap, net::SocketAddr, time::Duration};

use http::HttpRequest;
use serde::{Deserialize, Serialize};

pub mod http;
pub mod tcp;

#[derive(Debug)]
pub struct Monitor {
    pub service_data: MonitorData,
    pub service_name: String,
    pub interval_mins: u64,
    pub enabled: bool,
    pub timeout_secs: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MonitorData {
    Tcp {
        addr: SocketAddr,
        expected: tcp::TcpExpectedResponse,
    },
    Http {
        url: String, // url validity verified at creation time
        request: HttpRequest,
        expected: http::HttpExpectedResponse,
    },
}

pub enum MonitorResult {
    // Service responded with expected response
    // Contains the time it took to get the response and additional data such as bytes received if a wildcard is used
    // (response_time_ms, info)
    Ok(u128, String),
    // Service responded, but not with what was expected
    // Contains the response that we got
    // (response_time_ms, response)
    UnexpectedResponse(u128, String),
    // The server did not send a response or the port is firewalled
    // Could also be caused by packet loss
    // (info)
    Down(String),
    // An I/O error occurred while checking the service
    // (error)
    IoError(String),
}

impl MonitorData {
    // Running a service will execute the logic of the service and put its results into the database
    pub async fn run(&self, timeout_s: u16) -> MonitorResult {
        match self {
            Self::Tcp {
                addr,
                expected,
            } => tcp::tcp_service(addr, expected, Duration::from_secs(timeout_s as _)).await,
            Self::Http {
                url,
                expected,
                request,
            } => http::http_service(url, expected, Duration::from_secs(timeout_s as _), request).await,
        }
    }

    pub fn service_location_str(&self) -> String {
        match self {
            Self::Tcp { addr, .. } => {
                format!("tcp://{addr}")
            }
            Self::Http { url, .. } => url.to_string(),
        }
    }

    pub fn as_hashmap(&self) -> HashMap<String, String> {
        let mut hm = HashMap::new();
        match self {
            Self::Tcp { addr, expected } => {
                hm.insert("Socket Address".to_string(), addr.to_string());
                hm.insert("Expected Response".to_string(), format!("{expected:?}"));
            }
            Self::Http { url, request, expected } => {
                hm.insert("URL".to_string(), url.to_string());
                hm.insert("Method".to_string(), format!("{:?}", request.method));
                hm.insert("Headers".to_string(), format!("{:?}", request.headers.to_reqwest().unwrap()));
                let body = String::from_utf8(request.body.clone()).unwrap_or_else(|_| "binary".to_string());
                hm.insert("Body".to_string(), body);
                hm.insert("Expected response".to_string(), format!("{expected:?}"));
            }
        };

        hm
    }
}
