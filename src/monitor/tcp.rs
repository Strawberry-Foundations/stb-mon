use std::{net::SocketAddr, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::Instant,
};

use crate::octet::Octet;

use super::MonitorResult;

#[derive(Serialize, Deserialize, Debug)]
pub enum TcpExpectedResponse {
    // The connection does not get closed if we try to connect to the service
    OpenPort,
    // The server replies with specified bytes after sending the bytes
    // (bytes_sent, bytes_received)
    Bits(Vec<u8>, Vec<Octet>),
}

pub async fn tcp_service(
    addr: &SocketAddr,
    expected: &TcpExpectedResponse,
    timeout: Duration,
) -> MonitorResult {
    let start_time = Instant::now();
    let mut conn =
        match tokio::time::timeout(timeout, async { TcpStream::connect(addr).await }).await {
            Ok(Ok(conn)) => conn,
            Ok(Err(ioe)) => return MonitorResult::IoError(ioe.to_string()),
            Err(_) => return MonitorResult::ConnectionTimeout,
        };

    let (sent, expected) = match expected {
        TcpExpectedResponse::OpenPort => {
            return MonitorResult::Ok(
                Instant::now().duration_since(start_time).as_millis(),
                "The service successfully established the connection".to_string(),
            );
        }
        TcpExpectedResponse::Bits(x, y) => (x, y),
    };

    if let Err(ioe) = conn.write_all(&sent).await {
        return MonitorResult::IoError(ioe.to_string());
    };

    let mut buf = [0u8; 2048];
    let read = match tokio::time::timeout(timeout, conn.read(&mut buf)).await {
        Ok(Ok(n)) => n,
        Ok(Err(ioe)) => return MonitorResult::IoError(ioe.to_string()),
        Err(_) => return MonitorResult::ConnectionTimeout,
    };

    let bytes = buf[..read].to_vec();

    if expected.is_empty() {
        return MonitorResult::Ok(Instant::now().duration_since(start_time).as_millis(), format!("The service successfully established the connection and sent a response\n\nResponse hex: {bytes:x?}"));
    }

    todo!("Response hex pattern checking");
}
