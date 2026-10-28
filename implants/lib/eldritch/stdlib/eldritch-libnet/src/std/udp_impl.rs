use alloc::format;
use alloc::string::String;
use anyhow::Result;
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

async fn handle_udp(address: String, port: i32, data: String) -> Result<String> {
    let address_and_port = format!("{address}:{port}");

    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let _bytes_sent = sock.send_to(data.as_bytes(), address_and_port).await?;

    let mut response_buffer = [0; 1024];
    let (_bytes_copied, _addr) = sock.recv_from(&mut response_buffer).await?;

    let result_string =
        String::from(String::from_utf8(response_buffer.to_vec())?.trim_matches(char::from(0)));
    Ok(result_string)
}

pub fn udp(address: String, port: i32, data: String) -> Result<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(handle_udp(address, port, data))
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
        ));

        let (_a, actual_response) = tokio::join!(listen_task, send_task);
        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }
}
