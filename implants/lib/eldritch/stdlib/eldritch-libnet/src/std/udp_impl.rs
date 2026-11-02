use alloc::format;
use alloc::string::String;
use anyhow::Result;
use core::time::Duration;
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

pub const DEFAULT_TIMEOUT_SECS: u64 = 5;

fn resolve_timeout_secs(timeout: Option<i64>) -> Result<u64> {
    match timeout {
        None => Ok(DEFAULT_TIMEOUT_SECS),
        Some(t) if t > 0 => Ok(t as u64),
        Some(_) => Err(anyhow::anyhow!("timeout must be greater than 0")),
    }
}

async fn handle_udp(
    address: String,
    port: i32,
    data: String,
    timeout: Option<i64>,
) -> Result<String> {
    let timeout_secs = resolve_timeout_secs(timeout)?;
    let address_and_port = format!("{address}:{port}");

    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let _bytes_sent = sock.send_to(data.as_bytes(), address_and_port).await?;

    let mut response_buffer = [0; 1024];
    let timeout_duration = Duration::from_secs(timeout_secs);

    match tokio::time::timeout(timeout_duration, sock.recv_from(&mut response_buffer)).await {
        Ok(Ok((bytes_copied, _addr))) => {
            let result_string = String::from(
                String::from_utf8_lossy(&response_buffer[..bytes_copied])
                    .trim_matches(char::from(0)),
            );
            Ok(result_string)
        }
        // If there is no response (timeout) or the port is closed/unreachable, gracefully return ""
        Ok(Err(_)) | Err(_) => Ok(String::new()),
    }
}

pub fn udp(address: String, port: i32, data: String, timeout: Option<i64>) -> Result<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(handle_udp(address, port, data, timeout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;
    use tokio::task;

    async fn setup_udp_listener(
        address: String,
    ) -> anyhow::Result<(i32, task::JoinHandle<anyhow::Result<()>>)> {
        let sock = UdpSocket::bind(format!("{address}:0")).await?;
        let port: i32 = sock.local_addr().unwrap().port().into();
        let listener_handle = task::spawn(async move {
            let mut buf = [0; 1024];
            let mut i = 0;
            while i < 5 {
                let (bytes_copied, addr) = sock.recv_from(&mut buf).await?;
                let bytes_copied = sock.send_to(&buf[..bytes_copied], addr).await?;
                if bytes_copied > 1 {
                    break;
                }
                i += 1;
            }
            Ok(())
        });
        Ok((port, listener_handle))
    }

    #[tokio::test]
    async fn test_udp_send() -> anyhow::Result<()> {
        let expected_response = String::from("Hello world!");
        let (test_port, listen_task) = setup_udp_listener(String::from("127.0.0.1")).await?;

        let send_task = task::spawn(handle_udp(
            String::from("127.0.0.1"),
            test_port,
            expected_response.clone(),
            None,
        ));

        let (_a, actual_response) = tokio::join!(listen_task, send_task);
        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_udp_no_reply_timeout() -> anyhow::Result<()> {
        let sock = UdpSocket::bind("127.0.0.1:0").await?;
        let port: i32 = sock.local_addr().unwrap().port().into();
        let listener_handle = task::spawn(async move {
            let mut buf = [0; 1024];
            let _ = sock.recv_from(&mut buf).await;
            // Intentionally do not send any reply
            Ok::<(), anyhow::Error>(())
        });

        let send_task = task::spawn(handle_udp(
            String::from("127.0.0.1"),
            port,
            String::from("ping"),
            Some(1),
        ));

        let (_a, actual_response) = tokio::join!(listener_handle, send_task);
        assert_eq!("", actual_response.unwrap().unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_udp_unreachable_port() -> anyhow::Result<()> {
        let sock = UdpSocket::bind("127.0.0.1:0").await?;
        let port: i32 = sock.local_addr().unwrap().port().into();
        drop(sock);

        let response = handle_udp(
            String::from("127.0.0.1"),
            port,
            String::from("ping"),
            Some(1),
        )
        .await?;

        assert_eq!(response, "");
        Ok(())
    }

    #[test]
    fn test_udp_sync_wrapper_no_reply() {
        let result = udp(
            String::from("127.0.0.1"),
            65534,
            String::from("ping"),
            Some(1),
        );
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_udp_invalid_timeout() {
        assert!(
            udp(
                String::from("127.0.0.1"),
                65534,
                String::from("ping"),
                Some(0),
            )
            .is_err()
        );
        assert!(
            udp(
                String::from("127.0.0.1"),
                65534,
                String::from("ping"),
                Some(-1),
            )
            .is_err()
        );
    }
}
