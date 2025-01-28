use std::{io::ErrorKind, net::SocketAddr, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::Instant,
};

use super::MonitorResult;

#[derive(Serialize, Deserialize, Debug)]
pub enum TcpExpectedResponse {
    // The connection does not get closed if we try to connect to the service
    OpenPort,
    // The server replies with specified bytes after sending the bytes
    // (bytes_sent, bytes_received)
    Bits(Vec<u8>, String),
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
            Ok(Err(ioe)) => {
                if ioe.kind() == ErrorKind::ConnectionRefused {
                    return MonitorResult::Down("Server refused connection".to_string());
                }
                return MonitorResult::IoError(ioe.to_string());
            }
            Err(_) => return MonitorResult::Down("Connection timed out".to_string()),
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

    if let Err(ioe) = conn.write_all(sent).await {
        return MonitorResult::IoError(ioe.to_string());
    };

    let mut buf = [0u8; 2048];
    let read = match tokio::time::timeout(timeout, conn.read(&mut buf)).await {
        Ok(Ok(n)) => n,
        Ok(Err(ioe)) => return MonitorResult::IoError(ioe.to_string()),
        Err(_) => return MonitorResult::Down("Read timed out".to_string()),
    };

    let bytes = buf[..read].to_vec();

    if expected.is_empty() {
        return MonitorResult::Ok(
            Instant::now().duration_since(start_time).as_millis(),
            format!(
                "The service successfully established the connection and sent a response{}: {:x?}",
                if bytes.len() > 100 { " (trucated)" } else { "" },
                &bytes[..bytes.len().clamp(0, 100)],
            ),
        );
    }

    todo!("Response hex pattern checking");
}
