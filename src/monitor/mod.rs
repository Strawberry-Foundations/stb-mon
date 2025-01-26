use std::{net::SocketAddr, time::Duration};

use http::HttpRequest;
use serde::{Deserialize, Serialize};

pub mod http;
pub mod tcp;

pub struct Monitor {
    pub service_data: MonitorData,
    pub interval_mins: u64,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MonitorData {
    Tcp {
        addr: SocketAddr,
        expected: tcp::TcpExpectedResponse,
        timeout: Duration,
    },
    Http {
        url: String, // url validity verified at creation time
        request: HttpRequest,
        expected: http::HttpExpectedResponse,
        timeout: Duration,
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
    // (conn_refused)
    Down(bool),
    // An I/O error occurred while checking the service
    // (error)
    IoError(String),
}

impl MonitorData {
    // Running a service will execute the logic of the service and put its results into the database
    pub async fn run(&self) -> MonitorResult {
        match self {
            Self::Tcp {
                addr,
                expected,
                timeout,
            } => tcp::tcp_service(addr, expected, *timeout).await,
            Self::Http {
                url,
                expected,
                timeout,
                request,
            } => http::http_service(url, expected, *timeout, request).await,
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
}
