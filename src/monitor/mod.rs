use std::{net::SocketAddr, time::Duration};

use serde::{Deserialize, Serialize};

pub mod tcp;

#[derive(Serialize, Deserialize, Debug)]
pub enum Monitor {
    Tcp {
        addr: SocketAddr,
        expected: tcp::TcpExpectedResponse,
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
    ConnectionTimeout,
    // An I/O error occured while checking the service
    // (error)
    IoError(String),
}

impl Monitor {
    // Running a service will execute the logic of the service and put its results into the database
    pub async fn run(&self) -> MonitorResult {
        match self {
            Self::Tcp {
                addr,
                expected,
                timeout,
            } => tcp::tcp_service(addr, expected, *timeout).await,
        }
    }
}
